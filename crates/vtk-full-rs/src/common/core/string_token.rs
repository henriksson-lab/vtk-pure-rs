use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

/// VTK: `vtkStringToken::Hash`.
pub type StringTokenHash = u32;

/// VTK: `token_NAMESPACE::Invalid()`.
pub const INVALID_HASH: StringTokenHash = 0x811c9dc5;

const FNV_1A_PRIME: StringTokenHash = 0x01000193;

#[derive(Debug, Default)]
struct StringTokenManager {
    data: HashMap<StringTokenHash, String>,
    sets: HashMap<StringTokenHash, HashSet<StringTokenHash>>,
}

impl StringTokenManager {
    fn manage(&mut self, data: &str) -> StringTokenHash {
        let (id, _) = self.compute_internal(data);
        if id != INVALID_HASH {
            self.data.insert(id, data.to_string());
        }
        id
    }

    fn value(&self, id: StringTokenHash) -> String {
        self.data.get(&id).cloned().unwrap_or_default()
    }

    fn contains_data(&self, id: StringTokenHash) -> bool {
        self.data.contains_key(&id)
    }

    fn insert(&mut self, set: StringTokenHash, member: StringTokenHash) -> bool {
        if !self.data.contains_key(&set) || !self.data.contains_key(&member) {
            return false;
        }
        self.sets.entry(set).or_default().insert(member)
    }

    fn remove(&mut self, set: StringTokenHash, member: StringTokenHash) -> bool {
        if !self.data.contains_key(&member) {
            return false;
        }
        let Some(members) = self.sets.get_mut(&set) else {
            return false;
        };
        let did_remove = members.remove(&member);
        if members.is_empty() {
            self.sets.remove(&set);
        }
        did_remove
    }

    fn children(&self, set: StringTokenHash) -> HashSet<StringTokenHash> {
        self.sets.get(&set).cloned().unwrap_or_default()
    }

    fn all_groups(&self) -> HashSet<StringTokenHash> {
        self.sets.keys().copied().collect()
    }

    fn compute_internal(&self, data: &str) -> (StringTokenHash, bool) {
        let mut id = StringToken::string_hash(data);
        loop {
            match self.data.get(&id) {
                None => return (id, false),
                Some(existing) if existing == data => return (id, true),
                Some(_) => id = id.wrapping_add(1),
            }
        }
    }
}

fn manager() -> &'static Mutex<StringTokenManager> {
    static MANAGER: OnceLock<Mutex<StringTokenManager>> = OnceLock::new();
    MANAGER.get_or_init(|| Mutex::new(StringTokenManager::default()))
}

/// VTK: `vtkStringToken`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct StringToken {
    id: StringTokenHash,
}

impl StringToken {
    /// VTK: `vtkStringToken(const char* data = nullptr, std::size_t size = std::string::npos)`.
    pub const fn new() -> Self {
        Self { id: INVALID_HASH }
    }

    /// VTK: `vtkStringToken(const char* data, std::size_t size = std::string::npos)`.
    pub fn new_from_str(data: &str) -> Self {
        let id = manager()
            .lock()
            .expect("vtkStringToken manager")
            .manage(data);
        Self { id }
    }

    /// VTK: `vtkStringToken(const char* data, std::size_t size)`.
    pub fn new_from_str_with_len(data: &str, size: usize) -> Self {
        let end = size.min(data.len());
        let prefix = data
            .get(..end)
            .expect("vtkStringToken byte length must end on a UTF-8 boundary");
        Self::new_from_str(prefix)
    }

    /// VTK: `vtkStringToken(const std::string& data)`.
    pub fn new_from_string(data: &String) -> Self {
        Self::new_from_str(data)
    }

    /// VTK: `vtkStringToken(Hash tokenId)`.
    pub const fn from_hash(token_id: StringTokenHash) -> Self {
        Self { id: token_id }
    }

    /// VTK: `vtkStringToken::GetId`.
    pub const fn get_id(&self) -> StringTokenHash {
        self.id
    }

    /// VTK: `vtkStringToken::GetHash`.
    pub const fn get_hash(&self) -> u32 {
        self.id
    }

    /// VTK: `vtkStringToken::Data`.
    pub fn data(&self) -> String {
        manager()
            .lock()
            .expect("vtkStringToken manager")
            .value(self.id)
    }

    /// VTK: `vtkStringToken::IsValid`.
    pub const fn is_valid(&self) -> bool {
        self.id != INVALID_HASH
    }

    /// VTK: `vtkStringToken::HasData`.
    pub fn has_data(&self) -> bool {
        manager()
            .lock()
            .expect("vtkStringToken manager")
            .contains_data(self.id)
    }

    /// VTK: `vtkStringToken::StringHash`.
    pub fn string_hash(data: &str) -> StringTokenHash {
        Self::string_hash_bytes(data.as_bytes())
    }

    /// VTK: `token_NAMESPACE::Token::stringHash`.
    pub fn string_hash_bytes(data: &[u8]) -> StringTokenHash {
        data.iter().fold(INVALID_HASH, |hash, byte| {
            (hash ^ StringTokenHash::from(*byte)).wrapping_mul(FNV_1A_PRIME)
        })
    }

    /// VTK: `vtkStringToken::InvalidHash`.
    pub const fn invalid_hash() -> StringTokenHash {
        INVALID_HASH
    }

    /// VTK: `vtkStringToken::AddChild`.
    pub fn add_child(&self, member: StringToken) -> bool {
        if !self.is_valid() || !member.is_valid() {
            return false;
        }
        manager()
            .lock()
            .expect("vtkStringToken manager")
            .insert(self.id, member.id)
    }

    /// VTK: `vtkStringToken::RemoveChild`.
    pub fn remove_child(&self, member: StringToken) -> bool {
        if !self.is_valid() || !member.is_valid() {
            return false;
        }
        manager()
            .lock()
            .expect("vtkStringToken manager")
            .remove(self.id, member.id)
    }

    /// VTK: `vtkStringToken::Children`.
    pub fn children(&self, recursive: bool) -> HashSet<StringToken> {
        let mut result = HashSet::new();
        self.collect_children(recursive, &mut result);
        result
    }

    /// VTK: `vtkStringToken::AllGroups`.
    pub fn all_groups() -> HashSet<StringToken> {
        manager()
            .lock()
            .expect("vtkStringToken manager")
            .all_groups()
            .into_iter()
            .map(Self::from_hash)
            .collect()
    }

    fn collect_children(&self, recursive: bool, result: &mut HashSet<StringToken>) {
        let children = manager()
            .lock()
            .expect("vtkStringToken manager")
            .children(self.id);
        for child_id in children {
            let child = Self::from_hash(child_id);
            if recursive && !result.contains(&child) {
                child.collect_children(recursive, result);
            }
            result.insert(child);
        }
    }
}

impl Default for StringToken {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for StringToken {
    fn from(value: &str) -> Self {
        Self::new_from_str(value)
    }
}

impl From<String> for StringToken {
    fn from(value: String) -> Self {
        Self::new_from_str(&value)
    }
}

impl PartialOrd for StringToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringToken {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data().cmp(&other.data())
    }
}

impl PartialEq<str> for StringToken {
    fn eq(&self, other: &str) -> bool {
        self.data() == other
    }
}

impl PartialEq<StringToken> for str {
    fn eq(&self, other: &StringToken) -> bool {
        self == other.data()
    }
}

impl PartialEq<String> for StringToken {
    fn eq(&self, other: &String) -> bool {
        self.data() == *other
    }
}

impl PartialEq<StringToken> for String {
    fn eq(&self, other: &StringToken) -> bool {
        *self == other.data()
    }
}
