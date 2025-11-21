use crate::app::application::App;
use crate::extensions::{ParamValue, ParamsClone};
use crate::{debug_log, debug_trace};
use crate::connection::RequestContext;
use crate::url::parser::parse;
use std::future::Future;
use std::pin::Pin;
use std::slice::Iter;
use std::sync::Arc;
use crate::alias::PRwLock; 
// pub static ROOT_URL: OnceLock<Url> = OnceLock::new();
use super::super::app::middleware::*;
use super::pattern::PathPattern;

/// Represents a URL in the application.
/// This struct holds the various components of a URL, including its path, query parameters, and more.
pub struct Url<C: RequestContext> {
    // The last segment of the URL path
    path: PathPattern,

    // The child segments of the URL path
    // TODO: Replace PRwLock<Children<C>> with more granular locking strategy
    // Current design locks entire children collection during access, causing contention
    // Consider: Children<C> with internal DashMap or Arc<Url> with individual locks
    children: PRwLock<Children<C>>,

    // The ancestor segment of the URL path
    ancestor: PRwLock<Ancestor<C>>,

    // The handle method of the URL
    method: PRwLock<Option<Arc<dyn AsyncFinalHandler<C>>>>,

    // The middleware chain of the URL
    middlewares: PRwLock<Vec<Arc<dyn AsyncMiddleware<C>>>>,

    // The config of the URL
    params: PRwLock<ParamsClone>,

    // The step names of the URL path
    names: StepName,

    // Cached App reference to avoid repeated ancestor traversal
    app_cache: PRwLock<Option<Arc<App>>>,
}

pub struct Children<C: RequestContext> {
    // Private vec - only accessible through methods
    inner: Vec<Arc<Url<C>>>,
}

impl<C: RequestContext> Children<C> {
    /// Create a new empty Children collection
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Get a clone of the children vec for iteration (read-only access)
    pub fn get_vec(&self) -> Vec<Arc<Url<C>>> {
        self.inner.clone()
    }

    /// Check if there are any children
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the number of children
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Insert a child in priority order (Literal → Regex → Any → AnyPath)
    /// This is the only way to add children, ensuring proper ordering
    pub(crate) fn insert_ordered(&mut self, child: Arc<Url<C>>) {
        // Find insertion position based on priority
        let insert_pos = self.inner
            .iter()
            .position(|c| child.path.priority() < c.path.priority())
            .unwrap_or(self.inner.len());

        self.inner.insert(insert_pos, child);
    }

    /// Remove a child by pattern
    pub(crate) fn remove(&mut self, pattern: &PathPattern) -> Result<(), String> {
        if let Some(pos) = self.inner.iter().position(|c| c.path == *pattern) {
            self.inner.remove(pos);
            Ok(())
        } else {
            Err(format!("Child not found: {}", pattern))
        }
    }

    /// Find a child by pattern
    pub(crate) fn find(&self, pattern: &PathPattern) -> Option<Arc<Url<C>>> {
        self.inner.iter().find(|c| c.path == *pattern).cloned()
    }

    /// Format children for display purposes
    pub fn display_string(&self) -> String {
        let mut result = String::new();
        for child in self.inner.iter() {
            result.push_str(&format!("{}\n", child));
        }
        result
    }
}

pub enum Ancestor<C: RequestContext> {
    Nil,
    App(Arc<App>),
    Some(Arc<Url<C>>),
}

pub struct StepName(PRwLock<Vec<Option<String>>>);

impl StepName  {
    pub fn new() -> Self {
        StepName(PRwLock::new(vec![]))
    }

    pub fn get(&self, index: usize) -> Option<String> {
        let guard = self.0.read();
        guard.get(index).cloned().flatten()
    }

    pub fn index<A: AsRef<str>>(&self, name: A) -> Option<usize> {
        let guard = self.0.read();
        let name = name.as_ref();
        for (i, n) in guard.iter().enumerate() {
            if let Some(n) = n {
                if n == name {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl From<Vec<Option<String>>> for StepName {
    fn from(value: Vec<Option<String>>) -> Self {
        StepName(PRwLock::new(value))
    }
}

impl std::default::Default for StepName {
    fn default() -> Self {
        StepName::new()
    }
}

impl<C: RequestContext> std::fmt::Display for Url<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut func_str = String::new();
        // Look for whether the fuction is None or not
        if let Some(_) = self.method.read().as_ref() {
            func_str.push_str(&format!("Function Exists, "));
        } else {
            func_str.push_str("None, ");
        }
        let children_str = self.children.read().display_string();
        write!(f, "Url: {}, Function: {}, {{{}}}", self.path, func_str, children_str)
    }
}

impl<C: RequestContext + 'static> Url<C> {
    /// Create a new Url instance with all fields except cache (cache is auto-initialized)
    pub fn new(
        path: PathPattern,
        children: Children<C>,
        ancestor: Ancestor<C>,
        method: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
        params: ParamsClone,
        names: StepName,
    ) -> Self {
        Self {
            path,
            children: PRwLock::new(children),
            ancestor: PRwLock::new(ancestor),
            method: PRwLock::new(method),
            middlewares: PRwLock::new(middlewares),
            params: PRwLock::new(params),
            names,
            app_cache: PRwLock::new(None),
        }
    }

    pub async fn run(&self, mut rx: C) -> C {
        let final_handler = {
            let guard = self.method.read();
            guard.clone()
        };
        // Lock the middleware
        let middlewares = {
            let guard = self.middlewares.read();
            guard.clone()
        };
        // Runs the function inside it
        if let Some(method) = final_handler {
            run_chain(middlewares, method, rx).await
            // return method.handle(request).await;
        } else {
            rx.handle_error();
            rx
        }
    }

    /// Walk the URL tree based on the path segments.
    /// Returns Some(Arc<Self>) if a matching URL is found, otherwise None.
    /// Uses backtracking with priority ordering: Literal → Regex → Any → AnyPath
    ///
    /// # Security Note: URL Depth Validation Not Required
    ///
    /// This function does NOT validate maximum URL depth, and this is intentional, not a
    /// security vulnerability. Here's why depth limiting is unnecessary:
    ///
    /// ## URL Tree Structure is Programmer-Controlled
    /// - The URL routing tree is defined at compile-time/startup via the endpoint! macro
    /// - Routes are registered by developers, not end-users
    /// - Tree depth is bounded by the number of registered routes (finite and controlled)
    ///
    /// ## User Input Cannot Create Tree Nodes
    /// - User-provided URL paths are MATCHED against the fixed tree structure
    /// - The walk() function only traverses existing nodes, never creates new ones
    /// - Deeply nested user URLs simply return None (404), no recursion occurs
    ///
    /// ## Example
    /// ```ignore
    /// // Registered by programmer: /api/v1/users/{id}/posts/{post_id}  (depth: 5, fixed)
    ///
    /// // User request: /api/v1/users/123/posts/456          (matched successfully)
    /// // User request: /api/v1/users/123/posts/456/a/b/c/d  (returns None/404, safe)
    /// ```
    ///
    /// ## When Depth Limits Would Be Needed
    /// Its WIP and should be implemented in future. 
    /// TODO 
    /// 
    /// ## Actual DoS Protection
    /// The real recursion limit is at line 306 (MAX_DEPTH = 100 in app_with_depth), which
    /// protects against circular references in the ancestor chain, a different concern.
    pub fn walk<'a>(
        self: Arc<Self>,
        mut path: Iter<'a, &str>,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<Self>>> + Send + 'a>> {

        // Get the current segment to match
        let this_segment = match path.next() {
            Some(s) => *s,
            None => {
                // No more path segments - this node is the destination
                debug_trace!("walk: No more segments, returning self");
                return Box::pin(async move { Some(self) });
            }
        };

        debug_trace!("walk: Looking for segment: '{}'", this_segment);

        // Get children for iteration (cloned for async move)
        let children = {
            let guard = self.children.read();
            if guard.is_empty() {
                debug_trace!("walk: No children found, returning None");
                return Box::pin(async { None });
            }
            debug_trace!("walk: Found {} children", guard.len());
            guard.get_vec()
        };

        // Async portion: iterate through pre-ordered children with backtracking
        Box::pin(async move {
            // Children are already ordered by priority: Literal → Regex → Any → AnyPath
            for child_url in children.iter() {
                debug_trace!("walk: Comparing child path {:?} with segment '{}'", child_url.path, this_segment);

                // Check if this pattern matches the current segment
                if child_url.path.matches(this_segment) {
                    debug_trace!("walk: Pattern {:?} matches '{}'", child_url.path, this_segment);

                    // Try to walk deeper
                    if path.len() >= 1 {
                        // More segments to match - recurse
                        if let Some(result) = child_url.clone().walk(path.clone()).await {
                            debug_trace!("walk: Recursive walk succeeded");
                            return Some(result);
                        }
                        // Recursive walk failed - backtrack and try next child
                        debug_trace!("walk: Recursive walk failed, backtracking");
                    } else {
                        // No more segments - this child is the destination
                        debug_trace!("walk: Found final destination");
                        return Some(child_url.clone());
                    }
                }
            }

            debug_trace!("walk: No match found after trying all children");
            None
        })
    }

    pub async fn walk_str(self: Arc<Self>, path: &str) -> Arc<Url<C>> {
        let mut path = path.split('/').collect::<Vec<&str>>();
        path.remove(0);

        self.walk(path.iter()).await.unwrap_or_else(|| {
            // If no match is found, return a default URL
            dangling_url()
        })
    }

    /// Walk back through the ancestor and look for the APP
    /// Protected against infinite recursion with depth limit (max 100)
    /// Uses cache to avoid repeated traversal
    pub async fn app(&self) -> Result<Arc<App>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache_guard = self.app_cache.read();
            if let Some(app) = cache_guard.as_ref() {
                return Ok(app.clone());
            }
        }

        // Cache miss - traverse ancestor chain
        let app = self.app_with_depth(0).await?;

        // Update cache before returning
        *self.app_cache.write() = Some(app.clone());

        Ok(app)
    }

    /// Internal helper with depth tracking to prevent infinite recursion
    fn app_with_depth(&self, depth: usize) -> Pin<Box<dyn Future<Output = Result<Arc<App>, Box<dyn std::error::Error>>> + Send + '_>> {
        const MAX_DEPTH: usize = 100;

        Box::pin(async move {
            if depth > MAX_DEPTH {
                return Err(format!("Ancestor chain exceeds max depth {} - possible circular reference", MAX_DEPTH).into());
            }

            // Clone ancestor before await to drop the read guard
            let ancestor_clone = {
                let guard = self.ancestor.read();
                match &*guard {
                    Ancestor::Some(ancestor) => Some(ancestor.clone()),
                    Ancestor::App(app) => return Ok(app.clone()),
                    Ancestor::Nil => return Err("No ancestor found".into()),
                }
            }; // Guard dropped here

            // Now await with no lock held
            if let Some(ancestor) = ancestor_clone {
                ancestor.app_with_depth(depth + 1).await
            } else {
                unreachable!()
            }
        })
    }

    /// Set the ancestor of this URL to be the application
    pub fn set_app(&self, app: Arc<App>) {
        {
            let mut guard = self.ancestor.write();
            *guard = Ancestor::App(app.clone());
        }
        // Update cache when ancestor changes
        self.change_app_cache(app);
    }

    /// Internal method to update app cache for this URL and all descendants
    /// Called when ancestor is set to ensure cache consistency
    fn change_app_cache(&self, app: Arc<App>) {
        // Update this node's cache
        *self.app_cache.write() = Some(app.clone());

        // Recursively update all children's caches
        let children = {
            let guard = self.children.read();
            guard.get_vec()
        };

        for child in children.iter() {
            child.change_app_cache(app.clone());
        }
    }

    /// Retrieves a cloned value of type `T` from the URL's parameter storage.
    /// Returns `Some(T)` if the parameter exists and matches the type, `None` otherwise.
    pub fn get_params<T: ParamValue + Clone + 'static>(&self) -> Option<T> {
        let params = self.params.read();
        params.get::<T>().cloned()
    }

    /// Stores a value in the URL's parameter storage, overwriting any existing value
    /// of the same type. This only affects the current URL node, not its ancestors.
    pub fn set_params<T: ParamValue + 'static>(&self, value: T) {
        self.params.write().set(value);
    }

    /// Get the index of a segment by using its name
    pub fn match_seg_name_with_index<A: AsRef<str>>(&self, name: A) -> Option<usize> {
        self.names.index(name)
    }

    /// Runs the handler (if any) attached to this URL.
    /// If no handler exists, returns `NOT_FOUND`.
    pub fn run_child(
        self: Arc<Self>,
        mut rc: C,
    ) -> Pin<Box<dyn Future<Output = C> + Send>> {
        Box::pin(async move {
            let handler_opt = {
                let guard = self.method.read();
                guard.clone()
            };
            if let Some(handler) = handler_opt {
                return handler.handle(rc).await;
            } else {
                rc.handle_error();
                return rc;
            }
        })
    }

    /// Delete a child URL under this URL.
    /// If the child URL doesn't exist, it returns an error.
    /// # Arguments
    /// * `child` - The child URL to delete.
    /// # Returns
    /// * `Ok(())` - The child URL was deleted.
    /// * `Err(String)` - An error message.
    pub fn kill_child(self: &Arc<Self>, child: PathPattern) -> Result<(), String> {
        let mut guard = self.children.write();
        guard.remove(&child)
    }

    /// Creates a new child URL under this URL.
    /// If the child URL already exists, it deletes it first.
    /// If it doesn't exist, it creates a new one and returns it.
    /// # Arguments
    /// * `child` - The child URL to create.
    /// * `function` - The function to run when this URL is accessed. Wrapped in Option
    /// * `middleware` - The middleware to run when this URL is accessed. Wrapped in Option
    /// * `params` - The parameters to use for this URL. Wrapped in Option
    /// * `names` - The step names for this URL. Wrapped in Option
    /// # Returns
    /// * `Ok(Arc<Url>)` - The child URL.
    /// * `Err(String)` - An error message.
    /// # Note
    /// This function is not async, but it can be used in an async context.
    pub fn childbirth(
        self: &Arc<Self>,
        child: PathPattern,
        function: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middleware: Vec<Arc<dyn AsyncMiddleware<C>>>,
        params: ParamsClone,
        names: StepName,
    ) -> Result<Arc<Url<C>>, String> {
        debug_log!("Creating child URL: {:?}", child);

        // Check if child already exists - if so, update it in place
        if let Some(existing_child) = self.clone().find_child(&child) {
            // Update the existing child's properties (but keep its children)
            *existing_child.method.write() = function;
            *existing_child.middlewares.write() = middleware;
            *existing_child.params.write() = self.combine_params(&params);
            // Note: We don't update names or path as they define the child's identity
            return Ok(existing_child);
        }

        // Create new child if it doesn't exist
        let new_child = Arc::new(Url::new(
            child,
            Children::new(),
            Ancestor::Some(Arc::clone(&self)),
            function,
            middleware,
            self.combine_params(&params),
            names,
        ));

        // Add new child to parent in priority order
        let mut guard = self.children.write();
        guard.insert_ordered(new_child.clone());

        Ok(new_child)
    }

    pub fn default_url(self: &Arc<Self>, path: PathPattern) -> Arc<Self> {
        // Create a new URL with the default path
        Arc::new(Url::new(
            path,
            Children::new(),
            Ancestor::Nil,
            None,
            vec![],
            ParamsClone::new(),
            StepName::default(),
        ))
    }

    /// Get a child URL or create it if it doesn't exist.
    /// # Arguments
    /// * `child` - The child URL to get or create.
    /// # Returns
    /// * `Ok(Arc<Url>)` - The child URL.
    /// * `Err(String)` - An error message.
    /// # Note
    /// This function is not async, but it can be used in an async context.
    pub fn get_child_or_create(self: Arc<Self>, child: PathPattern) -> Result<Arc<Self>, String> {
        // Try to find existing child
        {
            let guard = self.children.read();
            if let Some(existing) = guard.find(&child) {
                return Ok(existing);
            }
        }
        // Child not found, create new one
        self.childbirth(child, None, vec![], ParamsClone::default(), StepName::default())
    }



    /// Register a child URL with a function.
    pub fn literal_url(
        self: Arc<Self>,
        path: &str,
        function: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middleware: Option<Vec<Arc<dyn AsyncMiddleware<C>>>>,
        params: ParamsClone,
    ) -> Result<Arc<Url<C>>, String> {
        debug_log!("Changing url into path pattern: {}", path);
        // Remove the first slash if exist
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        // Use register, convert the path to a Vec<PathPattern>
        let path_vec: Vec<PathPattern> = path.split('/').map(|s| PathPattern::literal_path(s)).collect();
        debug_log!("Path vector: {:?}", path_vec);
        // Call register with the path_vec and function
        let result = self.register(path_vec, function, middleware, params, StepName::default());
        // Return the result
        match result {
            Ok(url) => Ok(url),
            Err(e) => Err(format!("Error registering URL: {}", e)),
        }
    }

    // Using Hotaru pattern to register URL
    pub fn sub_url<A: AsRef<str>>(
        self: &Arc<Self>,
        path: A,
        function: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middleware: Option<Vec<Arc<dyn AsyncMiddleware<C>>>>,
        params: ParamsClone
    ) -> Result<Arc<Self>, String> {
        match parse(path.as_ref()) {
            Ok((path, names)) => {
                self.clone().register(
                    path,
                    function,
                    middleware,
                    params,
                    names.into()
                )
            },
            Err(e) => Err(format!("Error parsing child URL: {}", e)),
        }
    }

    /// Register a URL with a function.
    /// If the URL already exists, it updates the function.
    /// If middleware is None, it uses the URL you use to register's middleware.
    pub fn register(
        self: Arc<Self>,
        path: Vec<PathPattern>,
        function: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middleware: Option<Vec<Arc<dyn AsyncMiddleware<C>>>>,
        params: ParamsClone,
        names: StepName
    ) -> Result<Arc<Self>, String> {
        debug_log!("Registering URL: {:?}", path);
        let middleware = middleware.unwrap_or_else(|| (*self.middlewares.read()).clone());
        if path.len() == 0 {
            return self.childbirth(PathPattern::Literal("".to_string()), function, middleware, params, names);
        } else if path.len() == 1 {
            return self.childbirth(path[0].clone(), function, middleware, params, names);
        } else {
            debug_log!("Recursion: Registering child URL: {:?}", path[0]);
            return self.get_child_or_create(path[0].clone())?.register(path[1..].to_vec(), function, Some(middleware), params, names);
        }
    }

    /// Find a child URL with the given path pattern.
    /// Returns Some(Arc<Self>) if found, None otherwise.
    pub fn find_child(self: Arc<Self>, path: &PathPattern) -> Option<Arc<Self>> {
        let guard = self.children.read();
        guard.find(path)
    }

    pub fn set_method(&self, handler: Arc<dyn AsyncFinalHandler<C>>) {
        let mut guard = self.method.write();
        *guard = Some(handler);
    }

    pub fn set_middlewares(&self, middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>) {
        let mut guard = self.middlewares.write();
        *guard = middlewares;
    }

    /// Combine the current URL's parameters with the provided parameters.
    pub fn combine_params(&self, params: &ParamsClone) -> ParamsClone {
        let guard = self.params.read();
        let mut original = (*guard).clone();
        original.combine(params);
        return original
    }

    /// Merge the current URL's parameters with the provided parameters.
    pub fn merge_params(&self, params: &ParamsClone) -> ParamsClone {
        let guard = self.params.read();
        let mut original = (*guard).clone();
        original.combine(params);
        return original
    }

}

impl <C: RequestContext + 'static> Default for Url<C> {
    fn default() -> Self {
        Self::new(
            PathPattern::Literal(String::from("/")),
            Children::new(),
            Ancestor::Nil,
            None,
            vec![],
            ParamsClone::default(),
            StepName::default(),
        )
    }
}

pub fn dangling_url<C: RequestContext>() -> Arc<Url<C>> {
    Arc::new(Url::new(
        PathPattern::Any,
        Children::new(),
        Ancestor::Nil,
        None,
        vec![],
        ParamsClone::default(),
        StepName::default(),
    ))
}
