// Define the types
struct True;
struct False;

trait ChurchBool {
    type Select<T, F>;
    fn to_bool() -> bool;
}

impl ChurchBool for True {
    type Select<T, F> = T;
    fn to_bool() -> bool {
        true
    }
}

impl ChurchBool for False {
    type Select<T, F> = F;
    fn to_bool() -> bool {
        false
    }
}

// --- Logic Gates ---
type If<C, T, E> = <C as ChurchBool>::Select<T, E>;
type Not<B> = <B as ChurchBool>::Select<False, True>;
type And<P, Q> = <P as ChurchBool>::Select<Q, False>;
type Or<P, Q> = <P as ChurchBool>::Select<True, Q>;
type Xor<P, Q> = <P as ChurchBool>::Select<Not<Q>, Q>;

// --- Tuple Membership ---
trait Contains<Target> {
	type Output: ChurchBool;
}

impl<Target> Contains<Target> for () {
	type Output = False;
}

impl<Target, Tail> Contains<Target> for (Target, Tail) {
	type Output = True;
}
