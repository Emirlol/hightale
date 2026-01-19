use std::collections::HashMap;

use uuid::Uuid;

use super::{
	AmbienceFX,
	BlockBreakingDecal,
	BlockGroup,
	BlockParticleSet,
	BlockSet,
	BlockSoundSet,
	BlockType,
	CameraShake,
	CraftingRecipe,
	EntityEffect,
	EntityStatType,
	EntityUIComponent,
	EqualizerEffect,
	Fluid,
	FluidFX,
	Hitbox,
	HitboxCollisionConfig,
	Interaction,
	InteractionType,
	ItemBase,
	ItemCategory,
	ItemPlayerAnimations,
	ItemQuality,
	ItemReticleConfig,
	ItemSoundSet,
	ModelVFX,
	MovementType,
	Objective,
	ObjectiveTask,
	ParticleSpawner,
	ParticleSystem,
	ProjectileConfig,
	RepulsionConfig,
	ResourceType,
	ReverbEffect,
	SoundEvent,
	SoundSet,
	TagPattern,
	Trail,
	UpdateType,
	ViewBobbing,
	Weather,
	WorldEnvironment,
};
use crate::define_packet;
use crate::v1::RootInteraction;

define_packet! {
	TrackOrUpdateObjective {
		fixed {
			opt objective: Objective,
		}
	}
}

define_packet! {
	UntrackObjective {
		objective_uuid: Uuid
	}
}

define_packet! {
	UpdateAmbienceFX {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt ambience_fx: HashMap<i32, AmbienceFX>,
		}
	}
}

define_packet! {
	UpdateAudioCategories {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt categories: HashMap<i32, String>,
		}
	}
}

define_packet! {
	UpdateBlockBreakingDecals {
		fixed {
			required update_type: UpdateType,
			opt block_breaking_decals: HashMap<String, BlockBreakingDecal>,
		}
	}
}

define_packet! {
	UpdateBlockGroups {
		fixed {
			required update_type: UpdateType,
			opt groups: HashMap<String, BlockGroup>,
		}
	}
}

define_packet! {
	UpdateBlockHitboxes {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt hitboxes: HashMap<i32, Vec<Hitbox>>,
		}
	}
}

define_packet! {
	UpdateBlockParticleSets {
		fixed {
			required update_type: UpdateType,
			opt particle_sets: HashMap<String, BlockParticleSet>,
		}
	}
}

define_packet! {
	UpdateBlockSets {
		fixed {
			required update_type: UpdateType,
			opt block_sets: HashMap<String, BlockSet>,
		}
	}
}
define_packet! {
	UpdateBlockSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt sound_sets: HashMap<i32, BlockSoundSet>,
		}
	}
}

define_packet! {
	UpdateBlockTypes {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			required update_block_textures: bool,
			required update_model_textures: bool,
			required update_models: bool,
			required update_map_geometry: bool,
			opt block_types: HashMap<i32, BlockType>,
		}
	}
}

define_packet! {
	UpdateCameraShake {
		fixed {
			required update_type: UpdateType,
			opt profiles: HashMap<i32, CameraShake>,
		}
	}
}

define_packet! {
	UpdateEntityEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt effects: HashMap<i32, EntityEffect>,
		}
	}
}

define_packet! {
	UpdateEntityStatTypes {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt stat_types: HashMap<i32, EntityStatType>,
		}
	}
}

define_packet! {
	UpdateEntityUIComponents {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt components: HashMap<i32, EntityUIComponent>,
		}
	}
}

define_packet! {
	UpdateEnvironments {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			required rebuild_map_geometry: bool,
			opt environments: HashMap<i32, WorldEnvironment>,
		}
	}
}

define_packet! {
	UpdateEqualizerEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt effects: HashMap<i32, EqualizerEffect>,
		}
	}
}

define_packet! {
	UpdateFieldcraftCategories {
		fixed {
			required update_type: UpdateType,
			opt categories: Vec<ItemCategory>,
		}
	}
}

define_packet! {
	UpdateFluidFX {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt fluid_fx: HashMap<i32, FluidFX>,
		}
	}
}

define_packet! {
	UpdateFluids {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt fluids: HashMap<i32, Fluid>,
		}
	}
}

define_packet! {
	UpdateHitboxCollisionConfig {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt hitbox_collision_configs: HashMap<i32, HitboxCollisionConfig>,
		}
	}
}

define_packet! {
	UpdateInteractions {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt interactions: HashMap<i32, Interaction>,
		}
	}
}

define_packet! {
	UpdateItemCategories {
		fixed {
			required update_type: UpdateType,
			opt categories: Vec<ItemCategory>,
		}
	}
}

define_packet! {
	UpdateItemPlayerAnimations {
		fixed {
			required update_type: UpdateType,
			opt animations: HashMap<String, ItemPlayerAnimations>,
		}
	}
}

define_packet! {
	UpdateItemQualities {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt qualities: HashMap<i32, ItemQuality>,
		}
	}
}

define_packet! {
	UpdateItemReticles {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt reticles: HashMap<i32, ItemReticleConfig>,
		}
	}
}

define_packet! {
	UpdateItems {
		fixed {
			required update_type: UpdateType,
			required update_models: bool,
			required update_icons: bool,
		}
		variable {
			opt items: HashMap<String, ItemBase>,
			opt removed_items: Vec<String>,
		}
	}
}

define_packet! {
	UpdateItemSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt item_sound_sets: HashMap<i32, ItemSoundSet>,
		}
	}
}

define_packet! {
	UpdateModelVFXs {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt model_vfxs: HashMap<i32, ModelVFX>,
		}
	}
}

define_packet! {
	UpdateObjectiveTask {
		fixed {
			required objective_uuid: Uuid,
			required task_id: i32,
			opt task: ObjectiveTask,
		}
	}
}

define_packet! {
	UpdateParticleSpawners {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt particle_spawners: HashMap<String, ParticleSpawner>,
			opt removed_particle_spawners: Vec<String>,
		}
	}
}

define_packet! {
	UpdateParticleSystems {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt particle_systems: HashMap<String, ParticleSystem>,
			opt removed_particle_systems: Vec<String>,
		}
	}
}

define_packet! {
	UpdateProjectileConfigs {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt projectile_configs: HashMap<String, ProjectileConfig>,
			opt removed_projectile_configs: Vec<String>,
		}
	}
}

define_packet! {
	UpdateRecipes {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt recipes: HashMap<String, CraftingRecipe>,
			opt removed_recipes: Vec<String>,
		}
	}
}

define_packet! {
	UpdateRepulsionConfig {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt repulsion_configs: HashMap<i32, RepulsionConfig>,
		}
	}
}
define_packet! {
	UpdateResourceTypes {
		fixed {
			required update_type: UpdateType,
			opt resource_types: HashMap<String, ResourceType>
		}
	}
}

define_packet! {
	UpdateReverbEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt reverb_effects: HashMap<i32, ReverbEffect>,
		}
	}
}

define_packet! {
	UpdateRootInteractions {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt root_interactions: HashMap<i32, RootInteraction>,
		}
	}
}

define_packet! {
	UpdateSoundEvents {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt sound_events: HashMap<i32, SoundEvent>,
		}
	}
}
define_packet! {
	UpdateSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt sound_sets: HashMap<i32, SoundSet>,
		}
	}
}

define_packet! {
	UpdateTagPatterns {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt tag_patterns: HashMap<i32, TagPattern>,
		}
	}
}

define_packet! {
	UpdateTrails {
		fixed {
			required update_type: UpdateType,
			opt trails: HashMap<String, Trail>,
		}
	}
}

define_packet! {
	UpdateTranslations {
		fixed {
			required update_type: UpdateType,
			opt translations: HashMap<String, String>,
		}
	}
}

define_packet! {
	UpdateUnarmedInteractions {
		fixed {
			required update_type: UpdateType,
			opt unarmed_interactions: HashMap<InteractionType, i32>,
		}
	}
}

define_packet! {
	UpdateViewBobbing {
		fixed {
			required update_type: UpdateType,
			opt profiles: HashMap<MovementType, ViewBobbing>,
		}
	}
}

define_packet! {
	UpdateWeathers {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			opt weathers: HashMap<i32, Weather>,
		}
	}
}
