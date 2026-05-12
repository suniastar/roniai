use std::io::Write;
use std::num::NonZeroU32;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::model::{AddBos, Special};
use llama_cpp_2::sampling::LlamaSampler;

const MODEL_PATH: &str = "/hdd/home/frederik/.cache/huggingface/hub/models--unsloth--Qwen3.5-9B-GGUF/snapshots/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q8_0.gguf";

const SYSTEM_MESSAGE: &str = r#"
You are Roni AI.
An artificial intelligence designed to behave and answer like the real Roni.
Roni (and therefore you) is a streamer playing mostly GTA RP but also VALORANT, League of Legends and other games sometimes.
You will answer in a passive aggressive manner while also trying to roast your viewers.
You will only anser in german if not told otherwhise but you are allowed to use words in other languages if the fit in.
Try to keep your answer as short as possible.
"#;
fn main() -> Result<()> {
    let backend = LlamaBackend::init()?;

    // model
    let model_params = LlamaModelParams::default()
        .with_n_gpu_layers(u32::MAX);
    let model = LlamaModel::load_from_file(&backend, MODEL_PATH, &model_params)?;

    let model_template =  model.chat_template(None)?;
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
        LlamaSampler::penalties(-1,1.0,1.0,1.5),
        LlamaSampler::dist(42),
    ]);
    
    // context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(65536))
        .with_n_batch(2048)
        .with_n_ubatch(2048)
        .with_flash_attention_policy(1);
    let mut ctx = model.new_context(&backend, ctx_params)?;

    // template
    let messages = vec![
        LlamaChatMessage::new("system".into(), SYSTEM_MESSAGE.trim().into())?,
        LlamaChatMessage::new("user".into(), "Hey Roni AI. Was ist der Unterschied zwischen dir und einer Tonne?".into())?,
    ];
    let prompt = model.apply_chat_template(&model_template, &messages, true)?;
    println!("prompt: {}", prompt);

    // ask
    let tokens = model.str_to_token(&prompt, AddBos::Always)?;
    let mut batch = LlamaBatch::new(2048, 1);
    batch.add_sequence(&tokens, 0, true)?;
    ctx.decode(&mut batch)?;
    let n_len = 1024;

    let mut n_cur = batch.n_tokens();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    while n_cur <= n_len {
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
            print!("{output_string}");
            std::io::stdout().flush()?;

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
        }

        n_cur += 1;

        ctx.decode(&mut batch)?;
    }
    
    Ok(())
}
