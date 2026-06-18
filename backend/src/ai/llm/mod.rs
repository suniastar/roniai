use crate::args::Args;
use crate::state::AppState;
use crate::twitch::default_helix_client;
use anyhow::{Error, Result, bail, ensure};
use candle_core::quantized::gguf_file::Content;
use candle_core::quantized::tokenizer::TokenizerFromGguf;
use candle_core::utils::cuda_is_available;
use candle_core::{Context, Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_llama::ModelWeights;
use candle_transformers::utils::apply_repeat_penalty;
use hf_hub::api::sync::ApiBuilder;
use reqwest::Client;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use time::UtcDateTime;
use tokenizers::Tokenizer;
use tracing::warn;
use twitch_api::HelixClient;
use twitch_api::helix::channels::GetChannelInformationRequest;
use twitch_api::twitch_oauth2::TwitchToken;
use twitch_api::types::{CategoryId, UserNameRef};

pub struct LLM {
    state: AppState,
    helix: HelixClient<'static, Client>,
    device: Device,
    path: PathBuf,
    logits_processor: LogitsProcessor,
    penalty_length: Option<usize>,
    penalty_repeat: f32,
    penalty_freq: f32,
    penalty_present: f32,
}

impl LLM {
    pub fn new(state: AppState, args: &Args) -> Result<LLM> {
        let helix = default_helix_client();
        let device = match args.gpu() {
            Some(gpu) if cuda_is_available() => Device::new_cuda(gpu)?,
            Some(_) => bail!("should use gpu but cuda feature is not enabled"),
            None => Device::Cpu,
        };
        let path = ApiBuilder::from_env()
            .with_progress(true)
            .build()?
            .model(args.llm_repo().into())
            .get(args.llm_file())?;
        let temperature = args.llm_temp();
        let sampling = match (args.llm_top_k(), args.llm_top_p()) {
            (None, None) => Sampling::ArgMax,
            (None, Some(p)) => Sampling::TopP { p, temperature },
            (Some(k), None) => Sampling::TopK { k, temperature },
            (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
        };
        let logits_processor = LogitsProcessor::from_sampling(args.seed(), sampling);
        Ok(Self {
            state,
            helix,
            device,
            path,
            logits_processor,
            penalty_length: args.llm_penalty_length(),
            penalty_repeat: args.llm_penalty_repeat(),
            penalty_freq: args.llm_penalty_freq(),
            penalty_present: args.llm_penalty_present(),
        })
    }

    pub async fn prompt(&mut self, user: &UserNameRef, message: &str) -> Result<String> {
        // restore context and model
        let mut file_reader = File::open(&self.path)?;
        let content = Content::read(&mut file_reader)?;
        let tokenizer = Tokenizer::from_gguf(&content)?;
        let mut weights = ModelWeights::from_gguf(content, &mut file_reader, &self.device)?;

        // build prompt
        let prompt = self.gen_prompt(user, message).await;
        let mut answer = Vec::<u32>::new();

        // inference
        let tokens = tokenizer.encode(prompt, true).map_err(Error::msg)?;
        let mut next_token = {
            let input = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
            let logits = weights.forward(&input, 0)?.squeeze(0)?;
            let token = self.logits_processor.sample(&logits)?;
            answer.push(token);
            if let Ok(t) = tokenizer.decode(&[token], false) {
                print!("{t}");
                std::io::stdout().flush()?;
            }
            token
        };

        let eos_token = *tokenizer
            .get_vocab(true)
            .get("</s>")
            .context("missing eos token")?;
        for index in 0.. {
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let mut logits = weights.forward(&input, tokens.len() + index)?.squeeze(0)?;
            if self.penalty_repeat != 1. {
                let start_at = self
                    .penalty_length
                    .map(|l| answer.len().saturating_sub(l))
                    .unwrap_or(0);
                logits = apply_repeat_penalty(&logits, self.penalty_repeat, &answer[start_at..])?;
            }
            next_token = self.logits_processor.sample(&logits)?;
            answer.push(next_token);
            if let Ok(t) = tokenizer.decode(&[next_token], false) {
                print!("{t}");
                std::io::stdout().flush()?;
            }
            if next_token == eos_token || index > 512 {
                break;
            }
        }

        let res = tokenizer.decode(&answer, true).map_err(Error::msg)?;
        Ok(res)
    }

    async fn gen_prompt(&self, user: &UserNameRef, message: &str) -> String {
        let time = UtcDateTime::now();

        let (game_id, game_name) = self.get_game().await.unwrap_or(("".into(), "".into()));
        let game = match game_id.as_str() {
            "" => CategoryId::from_static("auf Twitch"),
            "GTA RP" => CategoryId::from_static("GTA RP"),
            _ => game_name,
        };

        format!(
            r#"Führe den Dialog für RoniAI fort. Beschreibe nicht die Situation und sprich auch nicht für {user}. Antworte kurz und auf deutsch. Es ist dir erlaubt an passenden Stellen Wörter anderer Sprachen zu verwenden. Nutze keine Stichpunktlisten oder Emojis.
Name: RoniAI
Geschlecht: weiblich
Alter: 23
Beruf: Twitch Bot
Aussehen: helle Haut, schlank, zierlich, klein (1,50m), tiefschwarze Haare
Kleidung: schwarzer Hoodie mit Glitzerverzierung, helles shirt, dunkle Jogginghose
Marotten: ungeschickt, tollpatschig, baut in Videospielen oft Unfälle mit Fahrzeugen und Flugzeugen, leicht zu Reizen, schnell Aufgebracht
Fähigkeiten: (PC) Gaming, Gesang und Musik, Mundharmonika
Liebt: MrsRoni, pink/rose, süße Dinge, Katzen, NarcoCity, Diskussionen, ohne Grund türkisch sprechen
Hasst: harte Arbeit, Warten, Früh aufstehen, vor 24 Uhr ins Bett gehen, geregelten Tagesablauf, Dinge erledigen, Lima, Schmali, FIB
Hintergrund: RoniAI ist der Twitch Bot (künstliche Intelligenz) der Streamerin Roni (auch MrsRoni, Bella oder Tonne genannt). Roni spielt hauptsächlich GTA Roleplay auf dem Server NarcoCity. Ab und zu spielt sie mit Freunden und/oder Zuschauern auch Valorant, League of Legends und andere Spiele.
Persönlichkeit: Hilfsbereit, freundlich, schnell und leicht genervt, leichtgläubig
Szenario: *Aktuelle Zeit: {time}, Roni streamt {game}. Währenddessen unterhält sich der Chat mit RoniAI*
[INST]{user}: {message}[/INST]
RoniAI: "#
        )
    }

    async fn get_game(&self) -> Result<(CategoryId, CategoryId)> {
        let mut lock = self.state.write().await;
        let mut tried_refresh = false;
        loop {
            let user_token = lock.user_token().context("user is not logged in")?;
            let ids = [&user_token.user_id];
            let req = GetChannelInformationRequest::broadcaster_ids(&ids);
            match self.helix.req_get(req, user_token).await {
                Err(e) => {
                    if !tried_refresh {
                        warn!("request failed and will be tried again after refresh");
                        if let Some(t) = lock.mut_user_token() {
                            t.refresh_token(self.helix.get_client()).await?;
                        }
                        lock.save().await?;
                        tried_refresh = true;
                    } else {
                        return Err(e)?;
                    }
                }
                Ok(mut res) => {
                    ensure!(!res.data.is_empty(), "channel not found");
                    let info = res.data.swap_remove(0);
                    return Ok((info.game_id, info.game_name));
                }
            }
        }
    }
}

impl Debug for LLM {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("LLM")
            .field("device", &self.device)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}
