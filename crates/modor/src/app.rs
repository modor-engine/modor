use crate::{platform, Node, RootNode};
use fxhash::FxHashMap;
use log::{debug, Level};
use std::any;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug)]
pub struct App {
    roots: FxHashMap<TypeId, RootNodeData>,
}

impl App {
    pub fn new<T>(log_level: Level) -> Self
    where
        T: RootNode,
    {
        platform::init_logging(log_level);
        debug!("initialize app...");
        let mut app = Self {
            roots: FxHashMap::default(),
        };
        app.root::<T>();
        debug!("app initialized");
        app
    }

    #[allow(clippy::needless_collect)]
    pub fn update(&mut self) {
        debug!("run update app...");
        let roots = self.roots.values().cloned().collect::<Vec<_>>();
        let mut ctx = Context { app: self };
        for root in roots {
            (root.update_fn)(root.value, &mut ctx);
        }
        debug!("app updated");
    }

    pub fn root<T>(&mut self) -> RefMut<'_, T>
    where
        T: RootNode,
    {
        let type_id = TypeId::of::<T>();
        let root = if self.roots.contains_key(&type_id) {
            self.retrieve_root::<T>(type_id)
        } else {
            self.create_root::<T>(type_id)
        };
        RefMut::map(root, Self::downcast_root)
    }

    fn create_root<T>(&mut self, type_id: TypeId) -> RefMut<'_, dyn Any>
    where
        T: RootNode,
    {
        let mut ctx = Context { app: self };
        debug!("create root node `{}`...", any::type_name::<T>());
        let root = RootNodeData::new(T::on_create(&mut ctx));
        debug!("root node `{}` created", any::type_name::<T>());
        self.roots.entry(type_id).or_insert(root).value.borrow_mut()
    }

    fn retrieve_root<T>(&mut self, type_id: TypeId) -> RefMut<'_, dyn Any> {
        self.roots
            .get_mut(&type_id)
            .expect("internal error: missing root node")
            .value
            .try_borrow_mut()
            .unwrap_or_else(|_| panic!("root node `{}` already borrowed", any::type_name::<T>()))
    }

    fn downcast_root<T>(value: &mut dyn Any) -> &mut T
    where
        T: Any,
    {
        value
            .downcast_mut::<T>()
            .expect("internal error: misconfigured root node")
    }
}

#[derive(Debug)]
pub struct Context<'a> {
    app: &'a mut App,
}

impl Context<'_> {
    pub fn root<T>(&mut self) -> RefMut<'_, T>
    where
        T: RootNode,
    {
        self.app.root()
    }
}

#[derive(Clone, Debug)]
pub struct RootNodeData {
    value: Rc<RefCell<dyn Any>>,
    update_fn: fn(Rc<RefCell<dyn Any>>, &mut Context<'_>),
}

impl RootNodeData {
    fn new<T>(value: T) -> Self
    where
        T: RootNode,
    {
        Self {
            value: Rc::new(RefCell::new(value)),
            update_fn: Self::update_root::<T>,
        }
    }

    fn update_root<T>(value: Rc<RefCell<dyn Any>>, ctx: &mut Context<'_>)
    where
        T: RootNode,
    {
        Node::update(
            value
                .borrow_mut()
                .downcast_mut::<T>()
                .expect("internal error: misconfigured root node"),
            ctx,
        );
    }
}
