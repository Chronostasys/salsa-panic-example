
#[salsa::jar(db = Db)]
pub struct Jar(
    Program,
    lowering,
    TrackedNode,
    lowering_node,
    TrackedNode2,
    lowering_node2
);

mod  db;
use std::{collections::{HashMap, BTreeMap}, cell::RefCell, rc::Rc};

use db::Database;
pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[salsa::input]
struct Program{
    node:DependencyNode
}


#[derive(Debug,Clone,Eq,PartialEq)]
struct DependencyNode{
    children:Vec<i64>,
    value:i64,
    id:i64,
    map:Rc< RefCell< BTreeMap<i64,TrackedNode>>>
}

unsafe impl Sync for DependencyNode {}
unsafe impl Send for DependencyNode {}


lazy_static::lazy_static!{
    static ref NODES:HashMap<i64,DependencyNode> = {
        let mut map = HashMap::new();
        let m = Rc::new(RefCell::new(BTreeMap::new()));
        map.insert(0,DependencyNode{
            children:vec![1],
            value:0,
            id:0,
            map:m.clone()

        });
        map.insert(1,DependencyNode{
            children:vec![2],
            value:1,
            id:1,
            map:m.clone()
        });
        map.insert(2,DependencyNode{
            children:vec![0], // if change this line to `children:vec![],` panic disappear
            value:2,
            id:2,
            map:m.clone()
        });

        map
    };
}


#[salsa::tracked]
fn lowering(db:&dyn Db, p:Program) ->() {
    let n = p.node(db).map.borrow_mut().entry(p.node(db).id).or_insert(TrackedNode::new(db, p.node(db))).clone();
    lowering_node(db, n)
}

#[salsa::input]
struct TrackedNode{
    node:DependencyNode
}


#[salsa::tracked]
struct TrackedNode2{
    node:DependencyNode
}

fn recover(db: &dyn Db,
    cycle: &salsa::Cycle,
    f: TrackedNode) {
    
}

#[salsa::tracked(recovery_fn=recover)]
fn lowering_node(db:&dyn Db, p:TrackedNode) ->() {
    lowering_node2(db, TrackedNode2::new(db, p.node(db)))

}
#[salsa::tracked]
fn lowering_node2(db:&dyn Db, p:TrackedNode2) ->() {
    for child in p.node(db).children.iter(){
        println!("child: {:?}",child);
        lowering(db, Program::new(db, NODES.get(child).unwrap().clone()));
    }

}


fn main() {
    let mut db = Database::default();
    let input = Program::new(&db, NODES.get(&0).unwrap().clone());
    lowering(& db, input);
    input.set_node(&mut db).to(NODES.get(&1).unwrap().clone());
    // panic here
    lowering(& db, input);
}