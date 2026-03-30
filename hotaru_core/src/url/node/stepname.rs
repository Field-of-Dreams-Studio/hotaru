use std::collections::HashMap;

pub struct StepName {
    pub inner: HashMap<String, usize>,
}

impl StepName {
    /// Creates an empty step-name map.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let names = StepName::new();
    /// assert!(names.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Returns whether there are no registered step names.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let names = StepName::new();
    /// assert!(names.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of registered step names.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("id", 0);
    /// assert_eq!(names.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Inserts or replaces a named step index.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// assert_eq!(names.insert("id", 1), None);
    /// assert_eq!(names.index("id"), Some(1));
    /// ```
    pub fn insert<A: Into<String>>(&mut self, name: A, index: usize) -> Option<usize> {
        self.inner.insert(name.into(), index)
    }

    /// Removes a named step index.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("id", 1);
    /// assert_eq!(names.remove("id"), Some(1));
    /// assert!(names.index("id").is_none());
    /// ```
    pub fn remove(&mut self, name: &str) -> Option<usize> {
        self.inner.remove(name)
    }

    /// Returns the path index for a named step.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("id", 1);
    /// assert_eq!(names.get("id"), Some(1));
    /// ```
    pub fn get(&self, name: &str) -> Option<usize> {
        self.inner.get(name).copied()
    }

    /// Returns the path index for a named step.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("slug", 2);
    /// assert_eq!(names.index("slug"), Some(2));
    /// ```
    pub fn index<A: AsRef<str>>(&self, name: A) -> Option<usize> {
        self.get(name.as_ref())
    }

    /// Returns whether a named step exists.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("slug", 2);
    /// assert!(names.contains("slug"));
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    /// Iterates over all named steps.
    ///
    /// ```rust
    /// use hotaru_core::url::node::StepName;
    ///
    /// let mut names = StepName::new();
    /// names.insert("id", 1);
    /// assert_eq!(names.iter().count(), 1);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&str, usize)> + '_ {
        self.inner.iter().map(|(name, index)| (name.as_str(), *index))
    }
}

impl Default for StepName {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, usize>> for StepName {
    fn from(inner: HashMap<String, usize>) -> Self {
        Self { inner }
    }
}

impl From<Vec<Option<String>>> for StepName {
    fn from(names: Vec<Option<String>>) -> Self {
        let mut inner = HashMap::new();
        for (index, name) in names.into_iter().enumerate() {
            if let Some(name) = name {
                inner.insert(name, index);
            }
        }
        Self { inner }
    }
}

impl Clone for StepName {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
