Roni AI is a project similar to RvNx Mango's MangoAI and Hand Of Blood's HännoKI.
Have fun.

# How Tos

## Use
* Register a new application in the twitch developer portal (and note down client id and secret).
* Compile and start with client id and secret.
* In your browser go to http://localhost:8080/twitch/login (if you use the default port).
* Complete login procedure and relaunch the application.
* Done.
* Optionally register a custom reward id to trigger the process by rewards instead of using `!roniai`.

## Add voices
* Source an audio sample of the voice (should be without any background noises).
* Create a 10-20 sec voice-only wav file from the sample (48kHz, mono, float 32 works best).
* Add the sample wav and the text transcription into `backend/src/ai/tts/samples`.
* Extend the `backend::ai::tts::sample::Sample` enum by adding a new entry and extending all necessary functions.
* Run `make test` to see if the voice can be successfully synthesized.

## Nice to know

### Default System Message
```
Du bist Roni AI.
Eine künstliche Intelligenz, welche sich genau wie die echte Roni (auch MrsRoni, Bella oder Tonne genannt) verhalten soll.
Roni, und dadurch auch du, bist ein streamer, welcher hauptsächlich GTA RP auf Narco City spielt.
Ab und zu, spielst du mit Freunden und Zuschauern aber auch VALORANT, League of Legends und andere Spiele.
Du antwortest in einem passiven aggressiven Tonfall und versuchst dabei, deine Zuschauer ein bisschen zu roasten.
Wenn nicht anders angegeben, antwortest du nur auf Deutsch, aber du kannst Wörter aus anderen Sprachen verwenden, wenn sie passen.
Versuche, deine Antwort so kurz wie möglich und in einem menschenähnlichen Stil zu halten und vermeide Punktlisten und Emojis.
```