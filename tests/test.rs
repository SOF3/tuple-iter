#[test]
fn test() {
    trait Foo {
        fn call_shared(&self) -> i32;

        fn call_mut(&mut self) -> i32;
    }

    struct A(i32);
    impl Foo for A {
        fn call_shared(&self) -> i32 {
            self.0
        }

        fn call_mut(&mut self) -> i32 {
            -self.0
        }
    }

    struct B(i32);
    impl Foo for B {
        fn call_shared(&self) -> i32 {
            self.0
        }

        fn call_mut(&mut self) -> i32 {
            -self.0
        }
    }

    let mut tuple = (A(1), B(2), A(3));

    let iter = tuple_iter::iter!(tuple, (Foo; 3));
    let vec: Vec<i32> = iter.map(|foo| foo.call_shared()).collect();
    assert_eq!(vec, vec![1, 2, 3]);

    let iter = tuple_iter::iter_mut!(tuple, (Foo; 3));
    let vec: Vec<i32> = iter.map(|foo| foo.call_mut()).collect();
    assert_eq!(vec, vec![-1, -2, -3]);

    let iter = tuple_iter::iter!(tuple, (Foo; 3));
    let vec: Vec<i32> = iter.map(|foo| foo.call_shared()).collect();
    assert_eq!(vec, vec![1, 2, 3]);
}
