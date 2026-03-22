# 🛡️ SENTINEL CORTEX: NATIVE GUI ARCHITECTURE (QUANTUM REVISION)

> **CLASSIFICATION:** TEKNOS-ALPHA // RUST NATIVE
> **PHILOSOPHY:** "If it consumes 1% CPU idle, it is too heavy."
> **AESTHETICS:** Cyber-Industrial / Brutalist Functionality / Neon Data Visualization.

---

## 1. 🧬 CORE PHILOSOPHY & "INTUITION" ENGINE

### 1.1 The "Intuition" Layer (Predictive UX)

Instead of static buttons, the GUI must anticipate intent based on system state.

* **Context-Aware Fab (Floating Action Button):**
  * *If Research is Running:* Turns into "Pause/Analyze Stream".
  * *If Operation 404s:* Turns into "Recover/Retry".
  * *If System Load > 90%:* Turns into "Kill Switch".
* **Neural Suggestions:** A dedicated sidebar panel that analyzes logs in real-time and suggests prompts (e.g., "Research output implies XZ Backdoor connection. Search for 'liblzma' next?").

### 1.2 The "Yatra" Performance Standard

* **Zero-Copy Rendering:** Use `wgpu` backend via `eframe` to render graphs directly on GPU.
* **Async-First:** No UI blocking. All heavy lifting (Vertex AI calls, Git ops, FFmpeg) moves to `tokio` threads communicating via `crossbeam` channels.
* **Memory Footprint:** Target < 50MB RAM idle.

---

## 2. 🏛️ MODULAR ARCHITECTURE (THE HEXAGON)

The GUI is divided into 6 "Vimanas" (Modules) accessible via a persistent Hot-Dock.

### 🟢 MODULE A: OPERATIONS NEXUS (The Watchtower)

* **Function:** Raw low-level monitoring of Vertex AI / GCS operations.
* **Deep Features:**
  * **Timeline View:** GANTT chart of parallel operations.
  * **Artifact Preview:** Hovering over a `gs://` URI prefetches a thumbnail.
  * **Cost Estimator:** Real-time calculation of Vertex AI costs.

### 🔵 MODULE B: RESEARCH LAB (The Oracle)

* **Function:** Interface for `sentinel research`.
* **Deep Features:**
  * **Node Graph Visualization (`egui_node_graph`):** Visualize connections between researched entities.
  * **Live Markdown:** Split-pane editor.
  * **Source-Tree:** Expandable tree view of analysis.

### 🔴 MODULE C: YOUTUBE FACTORY (The Forge)

* **Function:** Complete end-to-end Orchestration of Video Production.
* **CRITICAL FEATURE: GCLOUD QUEUE MONITOR**
  * **Pipeline Visualizer:** A Flowchart view showing the exact state of each video in the "Factory Line".
    * *State 1: Scripting (Gemini)*
    * *State 2: Voice Synthesis (Vertex TTS)*
    * *State 3: Video Generation (Veo/Imagen) [QUEUE DEPTH INDICATOR]*
    * *State 4: Stitching (Local FFmpeg)*
    * *State 5: Upload (YouTube API)*
  * **Queue Metrics:** Show "Estimated Time to Render" based on current position in the GCloud queue.
  * **Storyboard Grid:** Drag-and-drop ordering of video clips before stitching.
  * **Audio Waveform:** Visualizer for generated TTS audio.

### 🟡 MODULE D: SYSTEM MATRIX (The Nervous System)

* **Function:** `sentinel sysadmin` and Hardware Telemetry.
* **Deep Features:**
  * **Hexagonal Heatmap:** Visualizes CPU core load.
  * **Process Killer:** Filtered for `python`, `cargo`, `ffmpeg`.

### 🟣 MODULE E: NEURAL MEMORY (The Akashic Records)

* **Function:** RAG / Vector Database Interface.
* **Deep Features:**
  * **Memory Explorer:** Search the vector DB.
  * **Context Injector:** Drag-and-drop context loading.

### ⚛️ MODULE F: QUANTUM CORE (The Engine)

* **Function:** Visualizing the physics of the Sentinel/ME60OS kernel.
* **Deep Features:**
  * **Time Crystal Visualizer:**
    * **S60 Clock:** A rotating, multi-ring dial representing the Sumerian base-60 time flow.
    * **Phase Sync:** A visual "beat" indicator showing the synchronization status between Sentinel and ME60OS (Yatra Protocol).
  * **Quantum Simulation View:**
    * **Particle System:** `egui_plot` physics simulation representing the "state" of the AI's thought process (Entropy vs. Coherence).
    * **Wave Function Collapse:** Visual representation of decision-making probabilities.

---

## 3. 🎨 VISUAL LANGUAGE: "CYBER-INDUSTRIAL"

### 3.1 Palette (Tokyo Night Storm Modified)

* **Void:** `#1a1b26` (Background)
* **Panel:** `#24283b` (Secondary)
* **Data Cyan:** `#7dcfff` (Primary Accents)
* **Warning Amber:** `#e0af68`
* **Quantum Violet:** `#bb9af7` (Time Crystal / Physics)
* **Critical Neon:** `#f7768e` (Errors)

### 3.2 Typography & UI Elements

* **Font:** `JetBrains Mono` or `Terminus`.
* **Borders:** 1px solid, no shadows (Flat design).
* **Animations:** "Breathing" opacity for active tasks. "Spinning" rings for Time Crystal.

---

## 4. 🛠️ TECHNICAL IMPLEMENTATION PLAN

### Phase 1: Foundation

* [x] Basic `eframe` setup.
* [x] Ops Monitor (List View).
* [ ] Navigation Dock (6 Slots).

### Phase 2: Quantum & Factory (Priority)

* [ ] **Module F (Quantum):** Implement S60 Clock Logic in Rust (math-heavy).
* [ ] **Module C (Factory):** Build the "Pipeline Visualizer" for GCloud Queues. This requires aggregating status from `operations.json` into grouped stages.

### Phase 3: The "Deep" Integrations

* [ ] **Crates:** `sysinfo`, `egui_plot` (Graphs), `egui_extras` (Images), `egui_node_graph`.
* [ ] **Async Bridge:** EventBus.
