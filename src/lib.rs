use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

type Resolve<T> = dyn FnOnce(Option<T>) + Send;
type Reject<E> = dyn FnOnce(Option<E>) + Send;

pub struct InternalPromise<T, E> {
    result: Option<Result<T, E>>,
}

pub struct Promise<T, E> {
    internal: Arc<Mutex<InternalPromise<T, E>>>,
}


impl<T: 'static + Send, E: 'static + Send> InternalPromise<T, E> {
    fn new() -> Self {
        Self {
            result: None,
        }
    }

    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn resolve(&mut self, value: Option<T>) {
        self.result = value.map(Ok);
    }

    fn reject(&mut self, reason: Option<E>) {
        self.result = reason.map(Err);
    }
}

impl<T: 'static + Send, E: 'static + Send> Promise<T, E> {
    pub fn new<F>(executor: F) -> Self
    where
        F: FnOnce(Box<Resolve<T>>, Box<Reject<E>>)
    {
        let internal = Arc::new(Mutex::new(InternalPromise::new()));
        let resolver = internal.clone();
        let rejecter = internal.clone();

        executor(
            Box::new(move |value| {
                match resolver.lock() {
                    Ok(mut resolver) => {
                        resolver.resolve(value);
                    },
                    _ => panic!("Cannot lock resolver"),
                }
            }),
            Box::new(move |reason| {
                match rejecter.lock() {
                    Ok(mut rejecter) => {
                        rejecter.reject(reason);
                    },
                    _ => panic!("Cannot lock resolver"),
                }
            }),
        );

        Self {
            internal,
        }
    }

    pub fn is_done(&self) -> Result<bool, PoisonError<MutexGuard<'_, InternalPromise<T, E>>>> {
        self.internal.lock().map(|internal| internal.is_done())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn executes() {
        let mut called = false;
        let _ = Promise::<u32, u32>::new(|_, _| called = true);
        assert!(called);
    }

    #[test]
    fn resolves() {
        let p = Promise::<_, u32>::new(|resolve, _| {
            resolve(Some(1));
        });

        assert!(p.internal.lock().unwrap().result.is_some());
        assert!(p.internal.lock().unwrap().result.unwrap().is_ok());
        assert_eq!(p.internal.lock().unwrap().result.unwrap().unwrap(), 1);
    }

    #[test]
    fn rejects() {
        let p = Promise::<u32, _>::new(|_, reject| {
            reject(Some("some error"));
        });

        assert_eq!(p.internal.lock().unwrap().result.unwrap(), Err("some error"));
    }

    #[test]
    fn thread() {
        let mut handle = None;
        let p = Promise::<u32, u32>::new(|resolve, _| {
            handle = Some(std::thread::spawn(move || {
                resolve(Some(1));
            }));
        });

        // Flaky test
        // std::thread::sleep(std::time::Duration::from_millis(1000));

        assert_eq!(p.internal.lock().unwrap().result, None);

        handle.unwrap().join().unwrap();

        assert_eq!(p.internal.lock().unwrap().result, Some(Ok(1)));
    }
}
