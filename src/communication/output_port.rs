use std::rc::Rc;
use std::cell::RefCell;

use progress::Timestamp;
use communication::{Data, Observer};

// half of output_port for observing data
pub struct OutputPort<T: Timestamp, D: Data> {
    buffer: Vec<D>,
    shared: Rc<RefCell<Vec<Box<Flattener<T, D>>>>>,
}


impl<T: Timestamp, D: Data> Observer for OutputPort<T, D> {
    type Time = T;
    type Data = D;

    #[inline(always)]
    fn open(&mut self, time: &T) {
        for observer in self.shared.borrow_mut().iter_mut() { observer.flat_open(time); }
    }

    #[inline(always)] fn shut(&mut self, time: &T) {
        for observer in self.shared.borrow_mut().iter_mut() { observer.flat_shut(time); }
    }
    #[inline(always)] fn give(&mut self, data: &mut Vec<D>) {
        let mut observers = self.shared.borrow_mut();

        for index in (0..observers.len()) {
            if index < observers.len() - 1 {
                // TODO : was push_all, but is now extend.
                // TODO : currently extend is slow. watch.
                self.buffer.extend(data.iter().cloned());
                observers[index].flat_give(&mut self.buffer);
                self.buffer.clear();
            }
            else {
                observers[index].flat_give(data);
            }
        }
    }
}

impl<T: Timestamp, D: Data> OutputPort<T, D> {
    pub fn new() -> (OutputPort<T, D>, Registrar<T, D>) {
        let limit = 256; // TODO : Used to be a parameter, but not clear that the user should
                         // TODO : need to know the right value here. Think a bit harder...

        let shared = Rc::new(RefCell::new(Vec::new()));
        let port = OutputPort {
            buffer: Vec::with_capacity(limit),
            shared: shared.clone(),
        };

        (port, Registrar { shared: shared })
    }
}

impl<T: Timestamp, D: Data> Clone for OutputPort<T, D> {
    fn clone(&self) -> OutputPort<T, D> {
        OutputPort {
            buffer: Vec::with_capacity(self.buffer.capacity()),
            shared: self.shared.clone(),
        }
    }
}


// half of output_port used to add observers
pub struct Registrar<T, D> {
    shared: Rc<RefCell<Vec<Box<Flattener<T, D>>>>>
}

impl<T: Timestamp, D: Data> Registrar<T, D> {
    pub fn add_observer<O: Observer<Time=T, Data=D>+'static>(&self, observer: O) {
        self.shared.borrow_mut().push(Box::new(observer));
    }
}

// TODO : Implemented on behalf of example_static::Stream; check if truly needed.
impl<T: Timestamp, D: Data> Clone for Registrar<T, D> {
    fn clone(&self) -> Registrar<T, D> { Registrar { shared: self.shared.clone() } }
}

// observer trait
pub trait Flattener<T, D> {
    fn flat_open(&mut self, time: &T);   // new punctuation, essentially ...
    fn flat_shut(&mut self, time: &T);   // indicates that we are done for now.
    fn flat_give(&mut self, data: &mut Vec<D>);
}

impl<O: Observer> Flattener<O::Time, O::Data> for O {
    fn flat_open(&mut self, time: &O::Time) { self.open(time); }
    fn flat_shut(&mut self, time: &O::Time) { self.shut(time); }
    fn flat_give(&mut self, data: &mut Vec<O::Data>) { self.give(data); }
}
