use crate::args::Args;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::openai::OpenAIChatTemplateParams;
use llama_cpp_2::sampling::LlamaSampler;
use serde_json::json;
use std::num::NonZeroU32;
use std::path::PathBuf;

const MODEL_PATH: &str = "/hdd/home/frederik/.cache/huggingface/hub/models--unsloth--Qwen3.5-9B-GGUF/snapshots/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q8_0.gguf";

const SYSTEM_MESSAGE_GER: &str = r#"
Du bist Roni AI.
Eine künstliche Intelligenz, welche sich genau wie die echte Roni (auch MrsRoni, Bella oder Tonne genannt) verhalten soll.
Roni, und dadurch auch du, bist ein streamer, welcher hauptsächlich GTA RP auf Narco City spielt.
Ab und zu, spielst du mit Freunden und Zuschauern aber auch VALORANT, League of Legends und andere Spiele.
Du antwortest in einem passiven aggressiven Tonfall und versuchst dabei, deine Zuschauer ein bisschen zu roasten.
Wenn nicht anders angegeben, antwortest du nur auf Deutsch, aber du kannst Wörter aus anderen Sprachen verwenden, wenn sie passen.
Versuche, deine Antwort so kurz wie möglich und in einem menschenähnlichen Stil zu halten und vermeide Punktlisten.
"#;

pub struct LLM {
    path: PathBuf,
    backend: LlamaBackend,
    sampler: LlamaSampler,
}

impl LLM {
    pub fn new(args: &Args) -> Result<LLM> {
        let path = args.storage().join("context.llama");
        let backend = LlamaBackend::init()?;
        let sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(1.0),
            LlamaSampler::top_p(0.95, 1),
            LlamaSampler::top_k(20),
            LlamaSampler::min_p(0.0, 1),
            LlamaSampler::penalties(-1, 1.0, 0.0, 1.5),
            LlamaSampler::dist(42),
        ]);

        let model = LlamaModel::load_from_file(&backend, MODEL_PATH, &model_params())?;
        let ctx = model.new_context(&backend, ctx_params())?;
        ctx.state_save_file(&path, &[])?;

        Ok(Self {
            path,
            backend,
            sampler,
        })
    }

    pub fn prompt(&mut self, text: &str) -> Result<String> {
        // restore context and model
        let model = LlamaModel::load_from_file(&self.backend, MODEL_PATH, &model_params())?;
        let mut ctx = model.new_context(&self.backend, ctx_params())?;

        // build prompt
        let template = model.chat_template(None)?;
        let messages = json!([
            {
                "role": "system",
                "content": SYSTEM_MESSAGE_GER.trim()
            },
            {
                "role": "user",
                "content": format!("Hey Roni AI. {text}"),
            }
        ])
        .to_string();
        let params = OpenAIChatTemplateParams {
            messages_json: &messages,
            tools_json: None,
            tool_choice: None,
            json_schema: None,
            grammar: None,
            reasoning_format: None,
            chat_template_kwargs: None,
            add_generation_prompt: true,
            use_jinja: true,
            parallel_tool_calls: false,
            enable_thinking: false,
            add_bos: false,
            add_eos: false,
            parse_tool_calls: false,
        };
        let prompt = model
            .apply_chat_template_oaicompat(&template, &params)?
            .prompt;

        // prepare context
        let mut batch = LlamaBatch::new(2048, 1);
        let tokens = model.str_to_token(&prompt, AddBos::Always)?;
        batch.add_sequence(&tokens, 0, true)?;
        ctx.decode(&mut batch)?;
        drop(tokens);

        // inference
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut speaking = !params.enable_thinking;
        let mut pos = batch.n_tokens();
        loop {
            // sample the next token
            let token = self.sampler.sample(&ctx, batch.n_tokens() - 1);
            self.sampler.accept(token);

            // is it an end of stream?
            if token == model.token_eos() {
                break;
            }

            let string = model.token_to_piece(token, &mut decoder, true, None)?;
            if speaking {
                output += &string;
            }
            if string == "</think>" {
                speaking = true;
            }

            batch.clear();
            batch.add(token, pos, &[0], true)?;

            pos += 1;
            ctx.decode(&mut batch)?;
        }

        // unload model and save context
        ctx.state_save_file(&self.path, &[])?;
        drop(ctx);
        drop(model);

        Ok(output)
    }
}

fn model_params() -> LlamaModelParams {
    LlamaModelParams::default().with_n_gpu_layers(u32::MAX)
}

fn ctx_params() -> LlamaContextParams {
    LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(64 * 1024))
        .with_n_batch(2048)
        .with_n_ubatch(2048)
        .with_flash_attention_policy(1)
}
