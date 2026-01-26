use std::collections::HashMap;

use macros::define_packet;
use uuid::Uuid;

use crate::{
	codec::BoundedVarLen,
	v2::{
		interaction::InteractionType,
		AmbienceFX,
		AudioCategory,
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
		RootInteraction,
		SoundEvent,
		SoundSet,
		TagPattern,
		Trail,
		UpdateType,
		ViewBobbing,
		Weather,
		WorldEnvironment,
	},
};

define_packet! {
	TrackOrUpdateObjective {
		variable {
			opt(1) objective: Objective,
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
		}
		variable {
			opt(1) ambience_fx: HashMap<i32, AmbienceFX>,
		}
	}
}

define_packet! {
	UpdateAudioCategories {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) categories: HashMap<i32, AudioCategory>,
		}
	}
}

define_packet! {
	UpdateBlockBreakingDecals {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) block_breaking_decals: HashMap<String, BlockBreakingDecal>,
		}
	}
}

define_packet! {
	UpdateBlockGroups {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) groups: HashMap<String, BlockGroup>,
		}
	}
}

define_packet! {
	UpdateBlockHitboxes {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) hitboxes: HashMap<i32, BoundedVarLen<Vec<Hitbox>, 64>>,
		}
	}
}

define_packet! {
	UpdateBlockParticleSets {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) particle_sets: HashMap<String, BlockParticleSet>,
		}
	}
}

define_packet! {
	UpdateBlockSets {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) block_sets: HashMap<String, BlockSet>,
		}
	}
}
define_packet! {
	UpdateBlockSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) sound_sets: HashMap<i32, BlockSoundSet>,
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
		}
		variable {
			opt(1) block_types: HashMap<i32, BlockType>,
		}
	}
}

define_packet! {
	UpdateCameraShake {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) profiles: HashMap<i32, CameraShake>,
		}
	}
}

define_packet! {
	UpdateEntityEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) effects: HashMap<i32, EntityEffect>,
		}
	}
}

define_packet! {
	UpdateEntityStatTypes {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) stat_types: HashMap<i32, EntityStatType>,
		}
	}
}

define_packet! {
	UpdateEntityUIComponents {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) components: HashMap<i32, EntityUIComponent>,
		}
	}
}

define_packet! {
	UpdateEnvironments {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
			required rebuild_map_geometry: bool,
		}
		variable {
			opt(1) environments: HashMap<i32, WorldEnvironment>,
		}
	}
}

define_packet! {
	UpdateEqualizerEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) effects: HashMap<i32, EqualizerEffect>,
		}
	}
}

define_packet! {
	UpdateFieldcraftCategories {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) categories: Vec<ItemCategory>,
		}
	}
}

define_packet! {
	UpdateFluidFX {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) fluid_fx: HashMap<i32, FluidFX>,
		}
	}
}

define_packet! {
	UpdateFluids {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) fluids: HashMap<i32, Fluid>,
		}
	}
}

define_packet! {
	UpdateHitboxCollisionConfig {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) hitbox_collision_configs: HashMap<i32, HitboxCollisionConfig>,
		}
	}
}

define_packet! {
	UpdateInteractions {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) interactions: HashMap<i32, Interaction>,
		}
	}
}

define_packet! {
	UpdateItemCategories {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) categories: Vec<ItemCategory>,
		}
	}
}

define_packet! {
	UpdateItemPlayerAnimations {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) animations: HashMap<String, ItemPlayerAnimations>,
		}
	}
}

define_packet! {
	UpdateItemQualities {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) qualities: HashMap<i32, ItemQuality>,
		}
	}
}

define_packet! {
	UpdateItemReticles {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) reticles: HashMap<i32, ItemReticleConfig>,
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
			opt(1) items: HashMap<String, ItemBase>,
			opt(2) removed_items: Vec<String>,
		}
	}
}

define_packet! {
	UpdateItemSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) item_sound_sets: HashMap<i32, ItemSoundSet>,
		}
	}
}

define_packet! {
	UpdateModelVFXs {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) model_vfxs: HashMap<i32, ModelVFX>,
		}
	}
}

define_packet! {
	UpdateObjectiveTask {
		fixed {
			required objective_uuid: Uuid,
			required task_id: i32,
		}
		variable {
			opt(1) task: ObjectiveTask,
		}
	}
}

define_packet! {
	UpdateParticleSpawners {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) particle_spawners: HashMap<String, ParticleSpawner>,
			opt(2) removed_particle_spawners: Vec<String>,
		}
	}
}

define_packet! {
	UpdateParticleSystems {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) particle_systems: HashMap<String, ParticleSystem>,
			opt(2) removed_particle_systems: Vec<String>,
		}
	}
}

define_packet! {
	UpdateProjectileConfigs {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) projectile_configs: HashMap<String, ProjectileConfig>,
			opt(2) removed_projectile_configs: Vec<String>,
		}
	}
}

define_packet! {
	UpdateRecipes {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) recipes: HashMap<String, CraftingRecipe>,
			opt(2) removed_recipes: Vec<String>,
		}
	}
}

define_packet! {
	UpdateRepulsionConfig {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) repulsion_configs: HashMap<i32, RepulsionConfig>,
		}
	}
}
define_packet! {
	UpdateResourceTypes {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) resource_types: HashMap<String, ResourceType>
		}
	}
}

define_packet! {
	UpdateReverbEffects {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) reverb_effects: HashMap<i32, ReverbEffect>,
		}
	}
}

define_packet! {
	UpdateRootInteractions {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) root_interactions: HashMap<i32, RootInteraction>,
		}
	}
}

define_packet! {
	UpdateSoundEvents {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) sound_events: HashMap<i32, SoundEvent>,
		}
	}
}
define_packet! {
	UpdateSoundSets {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) sound_sets: HashMap<i32, SoundSet>,
		}
	}
}

define_packet! {
	UpdateTagPatterns {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) tag_patterns: HashMap<i32, TagPattern>,
		}
	}
}

define_packet! {
	UpdateTrails {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) trails: HashMap<String, Trail>,
		}
	}
}

define_packet! {
	UpdateTranslations {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) translations: HashMap<String, String>,
		}
	}
}

define_packet! {
	UpdateUnarmedInteractions {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) unarmed_interactions: HashMap<InteractionType, i32>,
		}
	}
}

define_packet! {
	UpdateViewBobbing {
		fixed {
			required update_type: UpdateType,
		}
		variable {
			opt(1) profiles: HashMap<MovementType, ViewBobbing>,
		}
	}
}

define_packet! {
	UpdateWeathers {
		fixed {
			required update_type: UpdateType,
			required max_id: i32,
		}
		variable {
			opt(1) weathers: HashMap<i32, Weather>,
		}
	}
}
