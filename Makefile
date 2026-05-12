help:
	# Shows any commands below with ## after the make target name as a list of available commands
	@echo "Available Commands:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

download-model: ## Download the required Qwen3.5 model
	echo todo

llama-qwen-35: ## Start the LLM server (Qwen3.5)
	llama-server \
		--ctx-size 65536 \
		--batch-size 2048 \
		--ubatch-size 2048 \
		--flash-attn on \
		--gpu-layers all \
		--hf-repo unsloth/Qwen3.5-9B-GGUF:Q4_K_M \
		--seed 42 \
		--temperature 1.0 \
		--top_k 20 \
		--top_p 0.95 \
		--min_p 0 \
		--repeat-penalty 1.0 \
		--presence-penalty 1.5 \
		--context-shift \
		--tools get_datetime \
		--jinja
