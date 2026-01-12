<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/logo.svg" alt="Drug Candidate Studio Logo" width="150"/>
</p>

<h1 align="center">💊 Drug Candidate Studio</h1>

<p align="center">
  <strong>A native Rust desktop application for computational drug discovery</strong><br>
  Molecular generation • Multi-objective optimization • Interactive visualization
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust" alt="Rust"/>
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License"/>
  <img src="https://img.shields.io/badge/Version-0.3.0-blue" alt="Version"/>
  <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey" alt="Platform"/>
</p>

---

## ✨ Key Features

<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/features.svg" alt="Features Overview" width="100%"/>
</p>

| Feature | Description |
|---------|-------------|
| 🧬 **Molecular Generation** | Generate drug-like molecules from 30+ real pharmaceutical scaffolds |
| 📊 **Multi-Objective Optimization** | Pareto front analysis for efficacy, toxicity, and costs |
| 💊 **Drug-likeness Rules** | Lipinski's Rule of Five, Veber rules, PAINS alerts |
| 🔬 **Similarity Analysis** | Tanimoto fingerprints, molecular clustering |
| 📈 **Advanced Visualization** | 3D plots, correlation heatmaps, histograms |
| ↩️ **Undo/Redo** | Full action history with 50+ levels |
| ⭐ **Annotations** | Notes and favorites on candidates |
| 🎨 **Themes** | Dark/Light mode with custom accent colors |
| 📥 **Import/Export** | SMILES, CSV, JSON, SDF formats |

---

## 🔄 Workflow

<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/workflow.svg" alt="Workflow" width="100%"/>
</p>

---

## 🧬 Molecular Generation

<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/molecule.svg" alt="Molecule Generation" width="500"/>
</p>

The application generates molecules using:
- **Real drug scaffolds**: Aspirin, Ibuprofen, Diazepam, Metformin, Caffeine, etc.
- **Hybrid generation**: Combining scaffold fragments with linkers
- **Random structures**: Novel molecular architectures

---

## 📊 Pareto Front Analysis

<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/pareto-front.svg" alt="Pareto Front" width="500"/>
</p>

Candidates on the **Pareto front** are "non-dominated" — no other candidate is better in *all* objectives simultaneously. These represent optimal trade-offs between:

| Objective | Direction | Description |
|-----------|-----------|-------------|
| **Efficacy** | Maximize ↑ | Drug effectiveness based on drug-likeness |
| **Toxicity** | Minimize ↓ | Predicted toxicity risk (PAINS alerts, LogP) |
| **Synthesis Cost** | Minimize ↓ | Chemical complexity |
| **Manufacturing Cost** | Minimize ↓ | Scale-up feasibility |

---

## 🏗️ Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/michaelgermini/Drug-Candidate-Studio/main/assets/architecture.svg" alt="Architecture" width="100%"/>
</p>

```
drug-candidate-studio/
├── Cargo.toml
├── assets/                    # SVG graphics
└── src/
    ├── main.rs               # Entry point
    ├── app/
    │   ├── mod.rs            # App structure
    │   ├── state.rs          # Application state & worker thread
    │   ├── history.rs        # Undo/Redo system
    │   ├── theme.rs          # Dark/Light themes
    │   ├── io.rs             # Import/Export
    │   └── ui/
    │       ├── top_bar.rs    # Menu & controls
    │       ├── side_panel.rs # Filters & details
    │       ├── candidates.rs # Main view & table
    │       ├── visualizations.rs  # Charts
    │       └── advanced_viz.rs    # 3D, heatmaps
    ├── chemistry/
    │   ├── descriptors.rs    # MW, LogP, PSA, HBD/HBA
    │   ├── smiles.rs         # SMILES generation
    │   ├── scaffolds.rs      # Drug templates (30+)
    │   ├── druglikeness.rs   # Lipinski, Veber, PAINS
    │   └── similarity.rs     # Tanimoto, clustering
    ├── generation/
    │   └── generator.rs      # Parallel generation
    └── optimization/
        ├── pareto.rs         # Pareto front algorithm
        └── objectives.rs     # Objective functions
```

---

## 🚀 Quick Start

### Requirements

- Rust 1.70+ with Cargo
- Windows, macOS, or Linux

### Build & Run

```bash
# Clone and navigate
cd drug-candidate-studio

# Run in release mode (recommended)
cargo run --release

# Or build only
cargo build --release
```

---

## 📖 Usage Guide

### 1️⃣ Generate Candidates

```
┌─────────────────────────────────────────┐
│  Generate: [300]  Seed: [42]  ⚡ 💊     │
│  [🧬 Generate]                          │
└─────────────────────────────────────────┘
```

- Set count and seed
- ⚡ = Parallel mode (uses all CPU cores)
- 💊 = Use pharmaceutical scaffolds

### 2️⃣ Explore & Filter

- **Pareto only**: Show optimal candidates
- **Favorites only**: Show starred items
- **SMILES search**: Find specific patterns
- **Range filters**: Efficacy/Toxicity bounds

### 3️⃣ Visualize

Enable from **View** menu:
- 📊 Histograms
- 📈 Parallel Coordinates
- 🎲 3D Plot
- 🔥 Correlation Heatmap
- 🔬 Clustering

### 4️⃣ Annotate

- Click ⭐ to favorite a candidate
- Add notes in the side panel
- All annotations are saved with sessions

### 5️⃣ Export

**File** menu offers:
- 💾 Save/Load Session (JSON)
- 📊 Export CSV
- 📋 Export JSON
- 🧬 Export SDF (chemistry software format)
- 📝 Export SMILES

---

## 🧪 Chemistry Module

The chemistry module provides comprehensive molecular analysis and generation capabilities, including property calculations, drug-likeness assessment, and similarity analysis.

### Molecular Descriptors

The application calculates key molecular properties from SMILES notation:

| Descriptor | Description | Calculation Method |
|------------|-------------|-------------------|
| **Molecular Weight (MW)** | Sum of atomic masses | Atomic mass lookup |
| **LogP** | Partition coefficient (lipophilicity) | Fragment-based estimation |
| **Polar Surface Area (PSA)** | Surface area of polar atoms | Simplified topological PSA |
| **H-bond Donors (HBD)** | Number of N-H, O-H bonds | Pattern matching |
| **H-bond Acceptors (HBA)** | Number of N, O atoms | Atom counting |
| **Rotatable Bonds** | Single bonds between non-terminal atoms | Bond topology analysis |

These descriptors are used to assess drug-likeness and calculate objective functions.

### Drug-likeness Rules

The application implements three major drug-likeness filters:

#### 📋 Lipinski's Rule of Five

The "Rule of Five" predicts oral bioavailability. A compound passes if it meets **3 out of 4** criteria:

| Criterion | Threshold | Rationale |
|-----------|-----------|-----------|
| **Molecular Weight** | ≤ 500 Da | Larger molecules have poor absorption |
| **LogP** | ≤ 5 | High lipophilicity reduces solubility |
| **H-bond Donors** | ≤ 5 | Too many HBD reduces membrane permeability |
| **H-bond Acceptors** | ≤ 10 | Excessive HBA reduces oral bioavailability |

**Example**: Aspirin (MW=180, LogP=1.2, HBD=1, HBA=4) ✅ **Passes all criteria**

#### 🔄 Veber Rules

Optimized for oral bioavailability prediction:

| Criterion | Threshold | Rationale |
|-----------|-----------|-----------|
| **Rotatable Bonds** | ≤ 10 | Flexibility affects oral absorption |
| **Polar Surface Area** | ≤ 140 Ų | High PSA reduces intestinal permeability |

**Example**: Metformin (RotBonds=0, PSA=68.5) ✅ **Excellent oral bioavailability**

#### ⚠️ PAINS (Pan-Assay Interference Compounds)

Detects **20+ problematic substructures** that cause false positives in assays:

| Alert Type | Examples | Severity |
|------------|----------|----------|
| **Reactive groups** | Epoxides, Michael acceptors, Acyl halides | High |
| **Frequent hitters** | Quinones, Rhodanines, Catechols | High |
| **Unstable groups** | Peroxides, Hydrazines, Disulfides | Medium |
| **Genotoxic alerts** | Nitro-aromatics, Azides, Nitroso | High |
| **Metabolic liabilities** | Anilines, Thioureas | Low |

**Example**: A molecule with `C1OC1` (epoxide) triggers a **High severity PAINS alert** ⚠️

### SMILES Generation

The application generates valid SMILES strings using multiple strategies:

1. **Scaffold-based generation** (60%): Uses real pharmaceutical templates
2. **Hybrid generation** (12%): Combines scaffold fragments with linkers
3. **Random generation** (28%): Creates novel molecular architectures

**Validation**: All generated SMILES are checked for:
- Balanced parentheses and brackets
- Valid ring closures
- Chemical valence rules
- Fallback to known valid SMILES if generation fails

### Similarity Analysis

#### Tanimoto Coefficient

Calculates molecular similarity using path-based fingerprints:

- **Fingerprint size**: 2048 bits
- **Features**: Atoms, atom pairs, 3-atom paths, functional groups, ring presence
- **Range**: 0.0 (dissimilar) to 1.0 (identical)

**Example**: 
- `CCO` (ethanol) vs `CCCO` (propanol): **Similarity ≈ 0.75**
- `c1ccccc1` (benzene) vs `O` (water): **Similarity ≈ 0.15**

#### Clustering

Groups similar molecules using the **leader algorithm**:
- Configurable similarity threshold (default: 0.5)
- Identifies cluster centroids
- Supports up to 200 molecules for performance

**Use cases**:
- Identify structurally similar candidates
- Reduce redundancy in candidate sets
- Explore chemical diversity

### Pharmaceutical Scaffolds

**30 real drug templates** organized by therapeutic category:

#### 💊 Analgesics & Anti-inflammatory (4)
- **Aspirin** - `CC(=O)Oc1ccccc1C(=O)O`
- **Ibuprofen** - `CC(C)Cc1ccc(cc1)C(C)C(=O)O`
- **Paracetamol** - `CC(=O)Nc1ccc(O)cc1`
- **Naproxen** - `COc1ccc2cc(ccc2c1)C(C)C(=O)O`

#### 💉 Antibiotics (3)
- **Penicillin-core** - `CC1(C)SC2C(NC(=O)C)C(=O)N2C1C(=O)O`
- **Sulfanilamide** - `Nc1ccc(cc1)S(N)(=O)=O`
- **Ciprofloxacin-core** - `c1cc2c(cc1F)c(=O)c(cn2C3CC3)C(=O)O`

#### ❤️ Cardiovascular (3)
- **Atenolol** - `CC(C)NCC(O)COc1ccc(cc1)CC(N)=O`
- **Propranolol** - `CC(C)NCC(O)COc1cccc2ccccc12`
- **Lisinopril-core** - `NCCCC(N)C(=O)N1CCCC1C(=O)O`

#### 🧠 Central Nervous System (4)
- **Diazepam-core** - `CN1C(=O)CN=C(c2ccccc2)c3cc(Cl)ccc13`
- **Fluoxetine** - `CNCCC(Oc1ccc(cc1)C(F)(F)F)c2ccccc2`
- **Sertraline-core** - `CNC1CCC(c2ccc(Cl)c(Cl)c2)c3ccccc13`
- **Caffeine** - `Cn1cnc2c1c(=O)n(c(=O)n2C)C`

#### 🤧 Antihistamines (2)
- **Diphenhydramine** - `CN(C)CCOC(c1ccccc1)c2ccccc2`
- **Loratadine-core** - `CCOC(=O)N1CCC(=C2c3ccc(Cl)cc3CCc4cccnc24)CC1`

#### 🩺 Antidiabetics (2)
- **Metformin** - `CN(C)C(=N)NC(=N)N`
- **Glipizide-core** - `Cc1cnc(cn1)C(=O)NCCc2ccc(cc2)S(=O)(=O)NC(=O)N`

#### 🦠 Antivirals (2)
- **Acyclovir** - `Nc1nc2c(ncn2COCCO)c(=O)[nH]1`
- **Oseltamivir-core** - `CCOC(=O)C1=CC(OC(CC)CC)C(NC(C)=O)C(N)C1`

#### 🎯 Anticancer (2)
- **Imatinib-core** - `Cc1ccc(NC(=O)c2ccc(CN3CCN(C)CC3)cc2)cc1Nc4nccc(n4)c5cccnc5`
- **Methotrexate-core** - `CN(Cc1cnc2nc(N)nc(N)c2n1)c3ccc(cc3)C(=O)NC(CCC(=O)O)C(=O)O`

#### 🧪 Common Heterocyclic Scaffolds (8)
- **Benzimidazole** - `c1ccc2[nH]cnc2c1`
- **Quinoline** - `c1ccc2ncccc2c1`
- **Indole** - `c1ccc2[nH]ccc2c1`
- **Pyrimidine** - `c1cncnc1`
- **Piperidine** - `C1CCNCC1`
- **Morpholine** - `C1COCCN1`
- **Piperazine** - `C1CNCCN1`
- **Thiazole** - `c1cscn1`
- **Oxazole** - `c1cocn1`
- **Triazole** - `c1cn[nH]n1`

---

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` | Native GUI framework |
| `egui_plot` | Plotting widgets |
| `rand` | Random number generation |
| `rayon` | Parallel processing |
| `serde` / `serde_json` | Serialization |
| `crossbeam-channel` | Thread communication |
| `chrono` | Timestamps |

---

## 🔮 Roadmap

- [ ] 🔬 RDKit integration for accurate calculations
- [ ] 🧠 ML model integration (ONNX)
- [ ] 📊 3D molecular visualization
- [ ] 🗄️ Database persistence (SQLite)
- [ ] 🔗 PubChem/ChEMBL integration
- [ ] 📱 Web version (WASM)

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ and 🦀 Rust
</p>
