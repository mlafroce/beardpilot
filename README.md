# BeardPilot
> A context-efficient CLI coding agent for local LLMs

BeardPilot is a minimalistic coding agent designed to work effectively with small context windows and locally running models via Ollama.

Inspired by PI Agent and similar approaches, with a stronger focus on constrained environments and local execution.

---

## Overview

Most coding agents assume:
- Large context windows
- Cloud-based models
- High latency tolerance

BeardPilot is built with different constraints in mind:
- Local LLMs (via Ollama)
- Limited context size
- Fast iteration and low overhead

The goal is to provide a simple, hackable CLI tool that remains useful even under tight resource limits.

---

## Quick Start

### 1. Install Ollama

```bash
curl -fsSL https://ollama.com/install.sh | sh
````

### 2. Run BeardPilot

```bash
cargo run
```

---

## Example Usage

```bash
beardpilot -p "Summarize the README file in 20 words. Add emojis."
```

### Example Output

```
BeardPilot is a context-efficient CLI agent for local LLMs via Ollama. Designed with small contexts and fast iterations in mind. 🐘🤖
```

---

## Design Goals

* **Context efficiency**
  Minimize token usage and avoid unnecessary prompt bloat.

* **Local-first**
  Designed specifically for models running on Ollama.

* **Simplicity**
  Keep the architecture understandable and easy to modify.

* **Hackability**
  Encourage experimentation and extension.

---

## Architecture (Work in Progress)

* Prompt builder optimized for small context windows
* Tool execution layer (e.g., bash, file system)
* Minimal or stateless interaction model
* Designed for extensibility

---

## Status

BeardPilot is in early development.

Current state:

* Basic CLI structure
* Initial Ollama integration (in progress)
* Tool execution system (in progress)

Planned:

* Context compression strategies
* Multi-step task handling

Not production ready.

---

## Configuration (Planned)

* Model selection (e.g., qwen, mistral)
* Context size tuning
* Tool permissions / sandboxing
* Prompt templates

---

## Roadmap

* [ ] Command execution layer
* [ ] Context optimization strategies
* [ ] File system awareness
* [ ] Plugin/tool system
* [ ] Multi-step planning
