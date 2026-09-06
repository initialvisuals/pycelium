//! 3D mesocosm. The organism is three-dimensional; a 2D view is a section.
//!
//! Soil mycelium forages in a pedon: litter at the surface, wetter pores
//! with depth, wood as elongated C-rich bodies, nitrogen in offset patches.
//! A plate photograph is a slice through that volume, not the volume.
//!
//! Layers:
//! 1. **Pedon (host RAM)** — organic / soluble C / soluble N / moisture.
//! 2. **Exoenzymes** — Michaelis–Menten cleavage, product repression, moisture gate.
//! 3. **Tips** — persistence + chemotropism + nitrotropism − autotropism.
//!    Extension is vesicle-limited. Lateral branches near 70°.
//! 4. **Network** — biomass wall. Anastomosis fuses a tip and donates cytoplasm.
//! 5. **Translocation** — internal C conducts only along biomass
//!    (conductance √(ρᵢρⱼ)). Tips are sinks; food is a source.
//!
//! Inoculum is a handful of germ tubes. Time, not particle count, fills the dish.

pub const GERM_TUBES_PER_SPORE: u32 = 72;
pub const INOCULUM_SITES: u32 = 5;
