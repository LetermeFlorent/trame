use std::collections::HashMap;

use fontdb::{Database, ID, Query, Source};

use super::discovery::{FontSource, SystemFonts};
use super::fallback::Fallbacks;
use super::{Face, Family, FontId};

pub struct FontLibrary {
    database: Database,
    faces: Vec<Face>,
    sources: HashMap<FontSource, Vec<FontId>>,
    shared: HashMap<ID, FontId>,
    scanned: bool,
    pub(super) fallbacks: Fallbacks,
}

impl FontLibrary {
    pub fn new(system: &SystemFonts) -> Self {
        Self {
            database: Database::new(),
            faces: Vec::new(),
            sources: HashMap::new(),
            shared: HashMap::new(),
            scanned: false,
            fallbacks: Fallbacks::new(&system.fallback, &system.emoji),
        }
    }

    pub fn face(&self, id: FontId) -> &Face {
        &self.faces[id.index()]
    }

    pub fn family(&mut self, sources: &[FontSource]) -> Family {
        Family::new(
            sources
                .iter()
                .flat_map(|source| self.load(source))
                .collect(),
        )
    }

    pub fn load(&mut self, source: &FontSource) -> Vec<FontId> {
        if let Some(ids) = self.sources.get(source) {
            return ids.clone();
        }
        let ids: Vec<ID> = self
            .database
            .load_font_source(Source::File(source.path.clone()))
            .into_iter()
            .filter(|id| {
                source
                    .index
                    .is_none_or(|index| self.face_index(*id) == Some(index))
            })
            .collect();
        let loaded: Vec<FontId> = ids.into_iter().filter_map(|id| self.share(id)).collect();
        self.sources.insert(source.clone(), loaded.clone());
        loaded
    }

    pub fn scan_generic(&mut self, generic: fontdb::Family<'_>) -> Family {
        if !self.scanned {
            self.database.load_system_fonts();
            self.scanned = true;
        }
        let families = [generic];
        let query = Query {
            families: &families,
            ..Query::default()
        };
        let Some(name) = self
            .database
            .query(&query)
            .and_then(|id| self.family_name(id))
        else {
            return Family::default();
        };
        let ids: Vec<ID> = self
            .database
            .faces()
            .filter(|face| {
                face.families
                    .first()
                    .is_some_and(|(family, _)| *family == name)
            })
            .map(|face| face.id)
            .collect();
        Family::new(ids.into_iter().filter_map(|id| self.share(id)).collect())
    }

    fn face_index(&self, id: ID) -> Option<u32> {
        self.database.face(id).map(|face| face.index)
    }

    fn family_name(&self, id: ID) -> Option<String> {
        let face = self.database.face(id)?;
        face.families.first().map(|(name, _)| name.clone())
    }

    fn share(&mut self, id: ID) -> Option<FontId> {
        if let Some(found) = self.shared.get(&id) {
            return Some(*found);
        }
        let next = FontId(u16::try_from(self.faces.len()).ok()?);
        let (data, index) = unsafe { self.database.make_shared_face_data(id) }?;
        self.faces.push(Face::new(data, index)?);
        self.shared.insert(id, next);
        Some(next)
    }
}
