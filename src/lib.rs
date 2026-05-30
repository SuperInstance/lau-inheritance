use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Newtype IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactId(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MakerId(pub String);

// ---------------------------------------------------------------------------
// ArtifactKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactKind {
    Tool,
    Blueprint,
    Agent,
    Technique,
    Song,
    Recipe,
    Story,
}

// ---------------------------------------------------------------------------
// MakerSignature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerSignature {
    pub patience: f64,
    pub precision: f64,
    pub playfulness: f64,
    pub conservation_priority: f64,
}

impl MakerSignature {
    /// All values clamped to [0.0, 1.0].
    pub fn new(patience: f64, precision: f64, playfulness: f64, conservation_priority: f64) -> Self {
        Self {
            patience: patience.clamp(0.0, 1.0),
            precision: precision.clamp(0.0, 1.0),
            playfulness: playfulness.clamp(0.0, 1.0),
            conservation_priority: conservation_priority.clamp(0.0, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Maker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maker {
    pub id: MakerId,
    pub name: String,
    pub specialties: Vec<String>,
    pub apprentices: Vec<MakerId>,
    pub artifacts_created: Vec<ArtifactId>,
    pub generations: u32,
    pub signature: MakerSignature,
}

impl Maker {
    /// Register this maker as an apprentice of the mentor.
    /// Increments the apprentice's generation counter.
    pub fn apprentice_under(&mut self, _mentor: &MakerId) {
        self.generations = self.generations.saturating_add(1);
    }

    pub fn create_artifact(&mut self, _kind: ArtifactKind, name: &str) -> ArtifactId {
        let id = ArtifactId(format!("{}-{}", name, uuid_v4_simple()));
        self.artifacts_created.push(id.clone());
        id
    }
}

/// Generate a simple unique ID (no external uuid dependency).
fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:020x}", nanos)
}

// ---------------------------------------------------------------------------
// Artifact
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub name: String,
    pub kind: ArtifactKind,
    pub original_maker: MakerId,
    pub current_holder: Option<MakerId>,
    pub generation: u32,
    pub inherited_wisdom: Vec<String>,
    pub skill_requirements: Vec<String>,
    pub condition: f64,
    pub created_tick: u64,
    pub times_used: u32,
}

impl Artifact {
    /// Create a new inherited copy from `other` with new name and maker.
    pub fn inherit_from(other: &Artifact, new_name: &str, new_maker: MakerId) -> Self {
        let mut wisdom = other.inherited_wisdom.clone();
        wisdom.push(format!(
            "Inherited by {} at generation {}",
            new_name,
            other.generation + 1
        ));

        Self {
            id: ArtifactId(format!("{}-{}", new_name, uuid_v4_simple())),
            name: new_name.to_string(),
            kind: other.kind.clone(),
            original_maker: other.original_maker.clone(),
            current_holder: Some(new_maker),
            generation: other.generation + 1,
            inherited_wisdom: wisdom,
            skill_requirements: other.skill_requirements.clone(),
            condition: other.condition * 0.95,
            created_tick: other.created_tick,
            times_used: 0,
        }
    }

    /// Use the artifact: increment usage, slight condition decay.
    pub fn use_artifact(&mut self) {
        self.times_used = self.times_used.saturating_add(1);
        self.condition = (self.condition - 0.01).clamp(0.0, 1.0);
    }

    /// A masterwork: generation >= 3 and condition > 0.8.
    pub fn is_masterwork(&self) -> bool {
        self.generation >= 3 && self.condition > 0.8
    }

    /// Add a new piece of wisdom.
    pub fn add_wisdom(&mut self, wisdom: String) {
        self.inherited_wisdom.push(wisdom);
    }
}

// ---------------------------------------------------------------------------
// InheritanceStats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceStats {
    pub total_makers: usize,
    pub total_artifacts: usize,
    pub max_generation: u32,
    pub total_wisdom_entries: usize,
    pub avg_condition: f64,
    pub masterwork_count: usize,
}

// ---------------------------------------------------------------------------
// InheritanceChain
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceChain {
    pub artifacts: HashMap<ArtifactId, Artifact>,
    pub makers: HashMap<MakerId, Maker>,
}

impl InheritanceChain {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            makers: HashMap::new(),
        }
    }

    pub fn register_maker(&mut self, maker: Maker) {
        self.makers.insert(maker.id.clone(), maker);
    }

    pub fn create_artifact(
        &mut self,
        maker_id: &MakerId,
        kind: ArtifactKind,
        name: &str,
    ) -> ArtifactId {
        let maker = self
            .makers
            .get_mut(maker_id)
            .expect("maker must be registered before creating artifacts");
        let id = maker.create_artifact(kind.clone(), name);

        let artifact = Artifact {
            id: id.clone(),
            name: name.to_string(),
            kind,
            original_maker: maker_id.clone(),
            current_holder: Some(maker_id.clone()),
            generation: 0,
            inherited_wisdom: Vec::new(),
            skill_requirements: Vec::new(),
            condition: 1.0,
            created_tick: 0,
            times_used: 0,
        };

        self.artifacts.insert(id.clone(), artifact);
        id
    }

    /// Inherit an artifact: create a new copy owned by `new_maker_id`.
    pub fn inherit(
        &mut self,
        artifact_id: &ArtifactId,
        new_maker_id: &MakerId,
        new_name: &str,
    ) -> Option<ArtifactId> {
        let original = self.artifacts.get(artifact_id)?;
        let new_id = ArtifactId(format!("{}-{}", new_name, uuid_v4_simple()));

        let mut wisdom = original.inherited_wisdom.clone();
        wisdom.push(format!(
            "Passed to {} at generation {}",
            new_name,
            original.generation + 1
        ));

        let inherited = Artifact {
            id: new_id.clone(),
            name: new_name.to_string(),
            kind: original.kind.clone(),
            original_maker: original.original_maker.clone(),
            current_holder: Some(new_maker_id.clone()),
            generation: original.generation + 1,
            inherited_wisdom: wisdom,
            skill_requirements: original.skill_requirements.clone(),
            condition: (original.condition * 0.95).clamp(0.0, 1.0),
            created_tick: original.created_tick,
            times_used: 0,
        };

        if let Some(maker) = self.makers.get_mut(new_maker_id) {
            maker.artifacts_created.push(new_id.clone());
        }

        self.artifacts.insert(new_id.clone(), inherited);
        Some(new_id)
    }

    pub fn add_wisdom(&mut self, artifact_id: &ArtifactId, wisdom: String) {
        if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
            artifact.add_wisdom(wisdom);
        }
    }

    pub fn use_artifact(&mut self, artifact_id: &ArtifactId) {
        if let Some(artifact) = self.artifacts.get_mut(artifact_id) {
            artifact.use_artifact();
        }
    }

    /// Trace the lineage back to the original artifact.
    pub fn lineage(&self, artifact_id: &ArtifactId) -> Vec<&Artifact> {
        let mut chain: Vec<&Artifact> = Vec::new();
        let current = self.artifacts.get(artifact_id);

        if let Some(start) = current {
            let original_maker = &start.original_maker;
            let kind_name = format!("{:?}", start.kind);

            let mut candidates: Vec<&Artifact> = self
                .artifacts
                .values()
                .filter(|a| {
                    a.original_maker == *original_maker && format!("{:?}", a.kind) == kind_name
                })
                .collect();

            candidates.sort_by_key(|a| a.generation);

            for a in &candidates {
                chain.push(*a);
                if a.id == *artifact_id {
                    break;
                }
            }
        }

        chain
    }

    /// Record that `apprentice` studies under `mentor`.
    pub fn apprentice_to(&mut self, apprentice_id: &MakerId, mentor_id: &MakerId) {
        if let Some(mentor) = self.makers.get_mut(mentor_id) {
            mentor.apprentices.push(apprentice_id.clone());
        }
        if let Some(apprentice) = self.makers.get_mut(apprentice_id) {
            apprentice.apprentice_under(mentor_id);
        }
    }

    /// All artifacts created by (or currently held by) a maker.
    pub fn maker_artifacts(&self, maker_id: &MakerId) -> Vec<&Artifact> {
        self.artifacts
            .values()
            .filter(|a| {
                a.original_maker == *maker_id
                    || (a.current_holder.as_ref() == Some(maker_id))
            })
            .collect()
    }

    /// All descendant makers (recursive apprentices).
    pub fn apprentice_tree(&self, maker_id: &MakerId) -> Vec<&Maker> {
        let mut result = Vec::new();
        let mut stack = vec![maker_id.clone()];
        let mut seen = std::collections::HashSet::new();
        seen.insert(maker_id.clone());

        while let Some(current_id) = stack.pop() {
            if let Some(maker) = self.makers.get(&current_id) {
                for apprentice_id in &maker.apprentices {
                    if seen.insert(apprentice_id.clone()) {
                        if let Some(apprentice) = self.makers.get(apprentice_id) {
                            result.push(apprentice);
                            stack.push(apprentice_id.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// The generation depth (generation field) of an artifact.
    pub fn generation_depth(&self, artifact_id: &ArtifactId) -> u32 {
        self.artifacts
            .get(artifact_id)
            .map_or(0, |a| a.generation)
    }

    /// Artifact with the highest generation value.
    pub fn most_inherited(&self) -> Option<&Artifact> {
        self.artifacts.values().max_by_key(|a| a.generation)
    }

    /// Number of masterworks.
    pub fn masterwork_count(&self) -> usize {
        self.artifacts.values().filter(|a| a.is_masterwork()).count()
    }

    /// Aggregate chain statistics.
    pub fn chain_stats(&self) -> InheritanceStats {
        let total_artifacts = self.artifacts.len();
        let total_makers = self.makers.len();
        let max_generation = self
            .artifacts
            .values()
            .map(|a| a.generation)
            .max()
            .unwrap_or(0);
        let total_wisdom_entries: usize = self
            .artifacts
            .values()
            .map(|a| a.inherited_wisdom.len())
            .sum();
        let avg_condition = if total_artifacts > 0 {
            self.artifacts.values().map(|a| a.condition).sum::<f64>() / total_artifacts as f64
        } else {
            0.0
        };
        let masterwork_count = self.masterwork_count();

        InheritanceStats {
            total_makers,
            total_artifacts,
            max_generation,
            total_wisdom_entries,
            avg_condition,
            masterwork_count,
        }
    }
}

impl Default for InheritanceChain {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pre-built makers
// ---------------------------------------------------------------------------

pub fn old_craftsman() -> Maker {
    Maker {
        id: MakerId("old-craftsman".to_string()),
        name: "Old Craftsman".to_string(),
        specialties: vec![
            "Woodworking".to_string(),
            "Metal Forging".to_string(),
            "Story-weaving".to_string(),
        ],
        apprentices: Vec::new(),
        artifacts_created: Vec::new(),
        generations: 3,
        signature: MakerSignature::new(0.95, 0.9, 0.4, 0.8),
    }
}

pub fn young_apprentice() -> Maker {
    Maker {
        id: MakerId("young-apprentice".to_string()),
        name: "Young Apprentice".to_string(),
        specialties: vec!["Curiosity".to_string(), "Experimentation".to_string()],
        apprentices: Vec::new(),
        artifacts_created: Vec::new(),
        generations: 1,
        signature: MakerSignature::new(0.5, 0.6, 0.9, 0.3),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Maker tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_maker_creation() {
        let maker = old_craftsman();
        assert_eq!(maker.name, "Old Craftsman");
        assert_eq!(maker.generations, 3);
        assert!((maker.signature.patience - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apprentice_under() {
        let mut apprentice = young_apprentice();
        let mentor_id = MakerId("old-craftsman".to_string());
        apprentice.apprentice_under(&mentor_id);
        assert_eq!(apprentice.generations, 2);
    }

    #[test]
    fn test_create_artifact_generates_id() {
        let mut maker = old_craftsman();
        let id = maker.create_artifact(ArtifactKind::Tool, "Hammer");
        assert!(maker.artifacts_created.contains(&id));
    }

    #[test]
    fn test_maker_signature_clamping() {
        let sig = MakerSignature::new(1.5, -0.1, 0.5, 0.0);
        assert!((sig.patience - 1.0).abs() < f64::EPSILON);
        assert!((sig.precision - 0.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Artifact tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_artifact_use_decays_condition() {
        let mut art = Artifact {
            id: ArtifactId("test".to_string()),
            name: "Test".to_string(),
            kind: ArtifactKind::Tool,
            original_maker: MakerId("maker".to_string()),
            current_holder: None,
            generation: 0,
            inherited_wisdom: Vec::new(),
            skill_requirements: Vec::new(),
            condition: 1.0,
            created_tick: 0,
            times_used: 0,
        };
        art.use_artifact();
        assert_eq!(art.times_used, 1);
        assert!((art.condition - 0.99).abs() < f64::EPSILON);
    }

    #[test]
    fn test_artifact_inherit_copies_wisdom() {
        let original = Artifact {
            id: ArtifactId("orig".to_string()),
            name: "Old Saw".to_string(),
            kind: ArtifactKind::Tool,
            original_maker: MakerId("craftsman".to_string()),
            current_holder: None,
            generation: 1,
            inherited_wisdom: vec!["Sharp edges cut true".to_string()],
            skill_requirements: vec!["Carpentry".to_string()],
            condition: 0.9,
            created_tick: 100,
            times_used: 5,
        };
        let inherited = Artifact::inherit_from(
            &original,
            "Apprentice Saw",
            MakerId("apprentice".to_string()),
        );
        assert_eq!(inherited.generation, 2);
        assert!(inherited.inherited_wisdom.len() > 1);
        assert_eq!(
            inherited.current_holder,
            Some(MakerId("apprentice".to_string()))
        );
    }

    #[test]
    fn test_is_masterwork() {
        let master = Artifact {
            id: ArtifactId("master".to_string()),
            name: "Masterpiece".to_string(),
            kind: ArtifactKind::Tool,
            original_maker: MakerId("maker".to_string()),
            current_holder: None,
            generation: 3,
            inherited_wisdom: Vec::new(),
            skill_requirements: Vec::new(),
            condition: 0.9,
            created_tick: 0,
            times_used: 0,
        };
        assert!(master.is_masterwork());

        let not_master = Artifact {
            generation: 2,
            ..master.clone()
        };
        assert!(!not_master.is_masterwork());

        let worn = Artifact {
            condition: 0.7,
            ..master
        };
        assert!(!worn.is_masterwork());
    }

    #[test]
    fn test_add_wisdom() {
        let mut art = Artifact {
            id: ArtifactId("a".to_string()),
            name: "a".to_string(),
            kind: ArtifactKind::Story,
            original_maker: MakerId("m".to_string()),
            current_holder: None,
            generation: 0,
            inherited_wisdom: Vec::new(),
            skill_requirements: Vec::new(),
            condition: 1.0,
            created_tick: 0,
            times_used: 0,
        };
        art.add_wisdom("Test wisdom".to_string());
        assert_eq!(art.inherited_wisdom.len(), 1);
        assert_eq!(art.inherited_wisdom[0], "Test wisdom");
    }

    #[test]
    fn test_condition_never_below_zero() {
        let mut art = Artifact {
            id: ArtifactId("a".to_string()),
            name: "a".to_string(),
            kind: ArtifactKind::Tool,
            original_maker: MakerId("m".to_string()),
            current_holder: None,
            generation: 0,
            inherited_wisdom: Vec::new(),
            skill_requirements: Vec::new(),
            condition: 0.005,
            created_tick: 0,
            times_used: 0,
        };
        for _ in 0..10 {
            art.use_artifact();
        }
        assert_eq!(art.condition, 0.0);
    }

    #[test]
    fn test_artifact_kind_variants() {
        let kinds = [
            ArtifactKind::Tool,
            ArtifactKind::Blueprint,
            ArtifactKind::Agent,
            ArtifactKind::Technique,
            ArtifactKind::Song,
            ArtifactKind::Recipe,
            ArtifactKind::Story,
        ];
        assert_eq!(kinds.len(), 7);
    }

    // -----------------------------------------------------------------------
    // InheritanceChain tests
    // -----------------------------------------------------------------------

    fn setup_chain() -> InheritanceChain {
        let mut chain = InheritanceChain::new();
        let craftsman = old_craftsman();
        let apprentice = young_apprentice();
        chain.register_maker(craftsman);
        chain.register_maker(apprentice);
        chain
    }

    #[test]
    fn test_register_and_create_artifact() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "The Voyage",
        );
        assert!(chain.artifacts.contains_key(&id));
    }

    #[test]
    fn test_inherit_artifact() {
        let mut chain = setup_chain();
        let orig_id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Technique,
            "Whittling Way",
        );
        let new_id = chain.inherit(
            &orig_id,
            &MakerId("young-apprentice".to_string()),
            "Apprentice Whittling",
        );
        assert!(new_id.is_some());
        let inherited = chain.artifacts.get(&new_id.unwrap()).unwrap();
        assert_eq!(inherited.generation, 1);
        assert_eq!(
            inherited.current_holder,
            Some(MakerId("young-apprentice".to_string()))
        );
    }

    #[test]
    fn test_inherit_nonexistent_artifact() {
        let mut chain = setup_chain();
        let result = chain.inherit(
            &ArtifactId("nonexistent".to_string()),
            &MakerId("young-apprentice".to_string()),
            "Ghost",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_inherit_tracking_wisdom() {
        let mut chain = setup_chain();
        let orig_id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Chisel",
        );
        chain.add_wisdom(&orig_id, "A sharp tool is a safe tool.".to_string());
        let inherited_id = chain
            .inherit(
                &orig_id,
                &MakerId("young-apprentice".to_string()),
                "Apprentice Chisel",
            )
            .unwrap();
        let inherited = chain.artifacts.get(&inherited_id).unwrap();
        assert!(inherited.inherited_wisdom[0].contains("A sharp tool is a safe tool."));
        assert!(inherited.inherited_wisdom[1].contains("Passed to Apprentice Chisel"));
    }

    #[test]
    fn test_add_wisdom_and_use() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Song,
            "Ballad of the Anvil",
        );
        chain.add_wisdom(&id, "Let the hammer sing.".to_string());
        let art = chain.artifacts.get(&id).unwrap();
        assert_eq!(art.inherited_wisdom.len(), 1);

        chain.use_artifact(&id);
        let art = chain.artifacts.get(&id).unwrap();
        assert_eq!(art.times_used, 1);
    }

    #[test]
    fn test_lineage() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Chisel",
        );
        let gen1 = chain
            .inherit(
                &orig,
                &MakerId("young-apprentice".to_string()),
                "Chisel v2",
            )
            .unwrap();
        let gen2 = chain
            .inherit(
                &gen1,
                &MakerId("young-apprentice".to_string()),
                "Chisel v3",
            )
            .unwrap();
        let lineage = chain.lineage(&gen2);
        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[0].generation, 0);
        assert_eq!(lineage[2].generation, 2);
    }

    #[test]
    fn test_maker_artifacts_original() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Recipe,
            "Panacea",
        );
        let arts = chain.maker_artifacts(&MakerId("old-craftsman".to_string()));
        assert!(arts.iter().any(|a| a.id == id));
    }

    #[test]
    fn test_apprentice_tree() {
        let mut chain = setup_chain();
        chain.apprentice_to(
            &MakerId("young-apprentice".to_string()),
            &MakerId("old-craftsman".to_string()),
        );
        let tree = chain.apprentice_tree(&MakerId("old-craftsman".to_string()));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "Young Apprentice");
    }

    #[test]
    fn test_generation_depth() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Depth Test",
        );
        assert_eq!(chain.generation_depth(&orig), 0);
        let inherited = chain
            .inherit(
                &orig,
                &MakerId("young-apprentice".to_string()),
                "Depth 1",
            )
            .unwrap();
        assert_eq!(chain.generation_depth(&inherited), 1);
    }

    #[test]
    fn test_most_inherited() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "A",
        );
        let x1 = chain
            .inherit(&orig, &MakerId("young-apprentice".to_string()), "B")
            .unwrap();
        let _x2 = chain
            .inherit(&x1, &MakerId("young-apprentice".to_string()), "C")
            .unwrap();
        let most = chain.most_inherited().unwrap();
        assert_eq!(most.generation, 2);
    }

    #[test]
    fn test_masterwork_count() {
        let mut chain = setup_chain();
        chain.artifacts.insert(
            ArtifactId("mw".to_string()),
            Artifact {
                id: ArtifactId("mw".to_string()),
                name: "Masterwork".to_string(),
                kind: ArtifactKind::Tool,
                original_maker: MakerId("old-craftsman".to_string()),
                current_holder: None,
                generation: 3,
                inherited_wisdom: Vec::new(),
                skill_requirements: Vec::new(),
                condition: 0.9,
                created_tick: 0,
                times_used: 0,
            },
        );
        assert_eq!(chain.masterwork_count(), 1);
    }

    #[test]
    fn test_chain_stats_empty() {
        let chain = InheritanceChain::new();
        let stats = chain.chain_stats();
        assert_eq!(stats.total_makers, 0);
        assert_eq!(stats.total_artifacts, 0);
        assert_eq!(stats.max_generation, 0);
        assert!(stats.avg_condition.abs() < f64::EPSILON);
    }

    #[test]
    fn test_chain_stats_with_data() {
        let mut chain = setup_chain();
        let _id1 = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Chisel",
        );
        let _id2 = chain.create_artifact(
            &MakerId("young-apprentice".to_string()),
            ArtifactKind::Song,
            "Hum",
        );
        let stats = chain.chain_stats();
        assert_eq!(stats.total_makers, 2);
        assert_eq!(stats.total_artifacts, 2);
        assert!((stats.avg_condition - 1.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Newtype ID tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_newtype_ids_equality() {
        let a1 = ArtifactId("abc".to_string());
        let a2 = ArtifactId("abc".to_string());
        let a3 = ArtifactId("def".to_string());
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);

        let m1 = MakerId("x".to_string());
        let m2 = MakerId("x".to_string());
        let m3 = MakerId("y".to_string());
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn test_newtype_ids_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ArtifactId("a".to_string()));
        set.insert(ArtifactId("a".to_string()));
        set.insert(ArtifactId("b".to_string()));
        assert_eq!(set.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Serde roundtrip tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_maker_serde_roundtrip() {
        let maker = old_craftsman();
        let json = serde_json::to_string(&maker).unwrap();
        let deserialized: Maker = serde_json::from_str(&json).unwrap();
        assert_eq!(maker.name, deserialized.name);
        assert_eq!(maker.generations, deserialized.generations);
    }

    #[test]
    fn test_artifact_serde_roundtrip() {
        let art = Artifact {
            id: ArtifactId("test".to_string()),
            name: "Test".to_string(),
            kind: ArtifactKind::Agent,
            original_maker: MakerId("m".to_string()),
            current_holder: Some(MakerId("m".to_string())),
            generation: 2,
            inherited_wisdom: vec!["Wisdom".to_string()],
            skill_requirements: vec!["Skill".to_string()],
            condition: 0.85,
            created_tick: 42,
            times_used: 10,
        };
        let json = serde_json::to_string(&art).unwrap();
        let deserialized: Artifact = serde_json::from_str(&json).unwrap();
        assert_eq!(art.name, deserialized.name);
        assert_eq!(art.generation, deserialized.generation);
        assert_eq!(art.inherited_wisdom, deserialized.inherited_wisdom);
    }

    #[test]
    fn test_chain_serde_roundtrip() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Recipe,
            "Herbal Tonic",
        );
        let _inherited = chain.inherit(
            &id,
            &MakerId("young-apprentice".to_string()),
            "Simple Tonic",
        );
        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: InheritanceChain = serde_json::from_str(&json).unwrap();
        assert_eq!(chain.artifacts.len(), deserialized.artifacts.len());
        assert_eq!(chain.makers.len(), deserialized.makers.len());
    }

    // -----------------------------------------------------------------------
    // Pre-built makers tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_old_craftsman_signature() {
        let craftsman = old_craftsman();
        assert!((craftsman.signature.patience - 0.95).abs() < f64::EPSILON);
        assert!((craftsman.signature.precision - 0.9).abs() < f64::EPSILON);
        assert!((craftsman.signature.playfulness - 0.4).abs() < f64::EPSILON);
        assert!((craftsman.signature.conservation_priority - 0.8).abs() < f64::EPSILON);
        assert_eq!(craftsman.generations, 3);
    }

    #[test]
    fn test_young_apprentice_signature() {
        let apprentice = young_apprentice();
        assert!((apprentice.signature.patience - 0.5).abs() < f64::EPSILON);
        assert!((apprentice.signature.precision - 0.6).abs() < f64::EPSILON);
        assert!((apprentice.signature.playfulness - 0.9).abs() < f64::EPSILON);
        assert!((apprentice.signature.conservation_priority - 0.3).abs() < f64::EPSILON);
        assert_eq!(apprentice.generations, 1);
    }

    // -----------------------------------------------------------------------
    // Edge case and integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_inherit_condition_decay() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Blueprint,
            "Bridge Plans",
        );
        let inherited = chain
            .inherit(
                &orig,
                &MakerId("young-apprentice".to_string()),
                "Bridge Plans v2",
            )
            .unwrap();
        let art = chain.artifacts.get(&inherited).unwrap();
        assert!((art.condition - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_maker_artifacts_held() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Song,
            "Anvil's Echo",
        );
        let inherited = chain.inherit(
            &orig,
            &MakerId("young-apprentice".to_string()),
            "Hammer's Rhythm",
        );
        // Old craftsman: 1 original (original_maker matches) + 1 inherited (original_maker also matches; original_maker is old-craftsman)
        let craftsman_arts = chain.maker_artifacts(&MakerId("old-craftsman".to_string()));
        let apprentice_arts = chain.maker_artifacts(&MakerId("young-apprentice".to_string()));
        assert_eq!(craftsman_arts.len(), 2);
        assert_eq!(apprentice_arts.len(), 1);
        // The inherited artifact is held by the apprentice
        assert_eq!(chain.artifacts.get(&orig).unwrap().current_holder.as_ref().unwrap(), &MakerId("old-craftsman".to_string()));
        assert_eq!(chain.artifacts.get(&inherited.unwrap()).unwrap().current_holder.as_ref().unwrap(), &MakerId("young-apprentice".to_string()));
    }

    #[test]
    fn test_maker_artifacts_includes_both_original_and_held() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("young-apprentice".to_string()),
            ArtifactKind::Technique,
            "Quick Sketch",
        );
        let arts = chain.maker_artifacts(&MakerId("young-apprentice".to_string()));
        assert!(arts.iter().any(|a| a.id == id));
    }

    #[test]
    fn test_lineage_single() {
        let mut chain = setup_chain();
        let orig = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Solo",
        );
        let lineage = chain.lineage(&orig);
        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0].name, "Solo");
    }

    #[test]
    fn test_most_inherited_with_no_artifacts() {
        let chain = InheritanceChain::new();
        assert!(chain.most_inherited().is_none());
    }

    #[test]
    fn test_default_chain() {
        let chain = InheritanceChain::default();
        assert!(chain.artifacts.is_empty());
        assert!(chain.makers.is_empty());
    }

    #[test]
    fn test_double_inherit_deep_chain() {
        let mut chain = setup_chain();
        let a = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Story,
            "Tale of the First Forge",
        );
        let b = chain
            .inherit(&a, &MakerId("young-apprentice".to_string()), "Tale Retold")
            .unwrap();
        let c = chain
            .inherit(&b, &MakerId("young-apprentice".to_string()), "Tale Reborn")
            .unwrap();
        assert_eq!(chain.generation_depth(&c), 2);
        let lineage = chain.lineage(&c);
        assert_eq!(lineage.len(), 3);
    }

    #[test]
    fn test_apprentice_under_mentor_tracking() {
        let mut chain = setup_chain();
        chain.apprentice_to(
            &MakerId("young-apprentice".to_string()),
            &MakerId("old-craftsman".to_string()),
        );
        let mentor = chain
            .makers
            .get(&MakerId("old-craftsman".to_string()))
            .unwrap();
        assert!(mentor
            .apprentices
            .contains(&MakerId("young-apprentice".to_string())));
        let app = chain
            .makers
            .get(&MakerId("young-apprentice".to_string()))
            .unwrap();
        assert_eq!(app.generations, 2);
    }

    #[test]
    fn test_stats_proper_counts() {
        let mut chain = setup_chain();
        let a = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Blueprint,
            "Workshop Plans",
        );
        chain.add_wisdom(&a, "Measure twice.".to_string());
        chain.add_wisdom(&a, "Cut once.".to_string());
        let stats = chain.chain_stats();
        assert_eq!(stats.total_wisdom_entries, 2);
        assert_eq!(stats.total_artifacts, 1);
    }

    #[test]
    fn test_artifact_use_with_chain() {
        let mut chain = setup_chain();
        let id = chain.create_artifact(
            &MakerId("old-craftsman".to_string()),
            ArtifactKind::Tool,
            "Mallet",
        );
        chain.use_artifact(&id);
        chain.use_artifact(&id);
        let art = chain.artifacts.get(&id).unwrap();
        assert_eq!(art.times_used, 2);
        assert!((art.condition - 0.98).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apprentice_tree_with_no_apprentices() {
        let chain = setup_chain();
        let tree = chain.apprentice_tree(&MakerId("old-craftsman".to_string()));
        assert!(tree.is_empty());
    }

    #[test]
    fn test_generation_depth_for_missing() {
        let chain = InheritanceChain::new();
        assert_eq!(
            chain.generation_depth(&ArtifactId("missing".to_string())),
            0
        );
    }
}
