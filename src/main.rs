mod embedded;

use crate::embedded::AudioSample;
use anyhow::{Result, ensure};
use candle_core::Tensor;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::openai::OpenAIChatTemplateParams;
use llama_cpp_2::sampling::LlamaSampler;
use serde_json::json;
use std::io::Write;
use std::num::NonZeroU32;
use std::time::Instant;
use qwen3_tts::{AudioBuffer, Qwen3TTS, SynthesisOptions, auto_device, Language};

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


fn main() -> Result<()> {
    let backend = LlamaBackend::init()?;

    // model
    let model_params = LlamaModelParams::default().with_n_gpu_layers(u32::MAX);
    let model = LlamaModel::load_from_file(&backend, MODEL_PATH, &model_params)?;

    let model_template = model.chat_template(None)?;
    println!("vocab size : {}", model.n_vocab());
    println!("context len: {}", model.n_ctx_train());
    println!("embed dim  : {}", model.n_embd());
    println!("template   : {:?}", model_template);

    // sampler
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(1.0),
        LlamaSampler::top_p(0.95, 1),
        LlamaSampler::top_k(20),
        LlamaSampler::min_p(0.0, 1),
        LlamaSampler::penalties(-1, 1.0, 0.0, 1.5),
        LlamaSampler::dist(42),
    ]);

    // context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(64 * 1024))
        .with_n_batch(2048)
        .with_n_ubatch(2048)
        .with_flash_attention_policy(1);
    let mut ctx = model.new_context(&backend, ctx_params)?;
    
    // template
    let messages = json!([
        {
            "role": "system",
            "content": SYSTEM_MESSAGE_GER.trim()
        },
        {
            "role": "user",
            "content": "Hey Roni AI. Kannst du mir sagen wie spät es ist?"
        }
    ])
        .to_string();
    let options = OpenAIChatTemplateParams {
        messages_json: &messages,
        tools_json: None,
        tool_choice: None,
        json_schema: None,
        grammar: None,
        reasoning_format: Some("auto"),
        chat_template_kwargs: Some(r#"{"enable_thinking": False}"#),
        add_generation_prompt: true,
        use_jinja: true,
        parallel_tool_calls: false,
        enable_thinking: true,
        add_bos: false,
        add_eos: false,
        parse_tool_calls: false,
    };
    let prompt = model
        .apply_chat_template_oaicompat(&model_template, &options)?
        .prompt;
    println!("prompt: {}", prompt);

    // ask
    let tokens = model.str_to_token(&prompt, AddBos::Always)?;
    let mut batch = LlamaBatch::new(2048, 1);
    batch.add_sequence(&tokens, 0, true)?;
    ctx.decode(&mut batch)?;

    let mut n_cur = batch.n_tokens();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut output = String::new();
    let mut speaking = !options.enable_thinking;
    loop {
        // sample the next token
        {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);

            sampler.accept(token);

            // is it an end of stream?
            if token == model.token_eos() {
                eprintln!();
                break;
            }

            let output_string = model.token_to_piece(token, &mut decoder, true, None)?;
            // use `Decoder.decode_to_string()` to avoid the intermediate buffer
            if speaking {
                output += &output_string;
            }
            if output_string == "</think>" {
                speaking = true;
            }
            print!("{output_string}");
            std::io::stdout().flush()?;

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
        }

        n_cur += 1;

        ctx.decode(&mut batch)?;
    }

    drop(ctx);
    drop(model);
    println!();
    println!();
    println!();
    println!("{}", output.trim());

    let device = auto_device()?;
    let model = Qwen3TTS::from_pretrained(
        "/hdd/home/frederik/ComfyUI/models/qwen-tts/Qwen3-TTS-12Hz-1.7B-Base",
        device,
    )?;

    // text
    let options = SynthesisOptions {
        seed: Some(42),
        ..SynthesisOptions::default()
    };

    // clone
    let ref_audio = AudioBuffer::load("src/embedded/roni_sample.wav")?;
    let ref_text = AudioSample::Roni.txt();
    ensure!(model.has_speech_encoder(), "speech encoder is required");

    let now = Instant::now();
    println!("start");
    let prompt = model.create_voice_clone_prompt(&ref_audio, Some(ref_text))?;
    println!("clone complete: {}ms", now.elapsed().as_millis());
    let audio  = model.synthesize_voice_clone(output.trim(), &prompt, Language::German, Some(options))?;
    println!("synthesize complete: {}ms", now.elapsed().as_millis());
    audio.save("output.wav")?;

    // Print semantic tokens to check for EOS (2150)
    eprintln!(
        "ICL clone: {:.2}s, {} samples → output.wav",
        audio.duration(),
        audio.len()
    );
    
    Ok(())
}
