use core::{
    ops::ControlFlow::{self, Break, Continue},
    task::Poll::{self, Pending, Ready},
};

type Never = core::convert::Infallible;

pub trait TryKind {
    type WithContinue<C>: CustomTry<Kind = Self, Continue = C>;
    type Residual;
    type Empty;
}

type KindOf<T> = <T as CustomTry>::Kind;
pub type ContinueOf<T> = <T as CustomTry>::Continue;
pub type ResidualOf<T> = <KindOf<T> as TryKind>::Residual;
pub type EmptyOf<T> = <KindOf<T> as TryKind>::Empty;
pub type WithContinue<T, C> = <KindOf<T> as TryKind>::WithContinue<C>;
pub type ControlFlowOf<T> = ControlFlow<ResidualOf<T>, Result<ContinueOf<T>, EmptyOf<T>>>;

pub trait CustomTry: Sized {
    type Continue;
    type Kind: TryKind<WithContinue<Self::Continue> = Self>;

    fn into_ctrl(self) -> ControlFlowOf<Self>;
    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self;
    fn from_continue(c: Self::Continue) -> Self {
        Self::from_ctrl(Continue(Ok(c)))
    }
    fn from_empty(e: EmptyOf<Self>) -> Self {
        Self::from_ctrl(Continue(Err(e)))
    }
    fn from_residual(b: ResidualOf<Self>) -> Self {
        Self::from_ctrl(Break(b))
    }
    fn map_continue<U>(self, f: impl FnOnce(Self::Continue) -> U) -> WithContinue<Self, U> {
        CustomTry::from_ctrl(match self.into_ctrl() {
            Continue(c) => Continue(c.map(f)),
            Break(r) => Break(r),
        })
    }
}

impl TryKind for Option<()> {
    type WithContinue<C> = Option<C>;
    type Residual = Option<Never>;
    type Empty = Never;
}

impl<T> CustomTry for Option<T> {
    type Kind = Option<()>;
    type Continue = T;

    fn into_ctrl(self) -> ControlFlowOf<Self> {
        match self {
            Some(c) => Continue(Ok(c)),
            None => Break(None),
        }
    }

    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self {
        match ctrl {
            Continue(Ok(c)) => Some(c),
            Break(None) => None,
        }
    }
}

impl<E> TryKind for Result<(), E> {
    type WithContinue<C> = Result<C, E>;
    type Residual = Result<Never, E>;
    type Empty = Never;
}

impl<T, E> CustomTry for Result<T, E> {
    type Kind = Result<(), E>;
    type Continue = T;

    fn into_ctrl(self) -> ControlFlowOf<Self> {
        match self {
            Ok(c) => Continue(Ok(c)),
            Err(b) => Break(Err(b)),
        }
    }

    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self {
        match ctrl {
            Continue(Ok(c)) => Ok(c),
            Break(Err(b)) => Err(b),
        }
    }
}

impl<B> TryKind for ControlFlow<B> {
    type WithContinue<C> = ControlFlow<B, C>;
    type Residual = ControlFlow<B, Never>;
    type Empty = Never;
}

impl<C, B> CustomTry for ControlFlow<B, C> {
    type Kind = ControlFlow<B>;
    type Continue = C;

    fn into_ctrl(self) -> ControlFlowOf<Self> {
        match self {
            Continue(c) => Continue(Ok(c)),
            Break(b) => Break(Break(b)),
        }
    }

    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self {
        match ctrl {
            Continue(Ok(c)) => Continue(c),
            Break(Break(b)) => Break(b),
        }
    }
}

impl<E> TryKind for Poll<Result<(), E>> {
    type WithContinue<C> = Poll<Result<C, E>>;
    type Residual = Result<Never, E>;
    type Empty = Poll<Never>;
}

impl<T, E> CustomTry for Poll<Result<T, E>> {
    type Kind = Poll<Result<(), E>>;
    type Continue = T;

    fn into_ctrl(self) -> ControlFlowOf<Self> {
        match self {
            Ready(Ok(x)) => Continue(Ok(x)),
            Ready(Err(e)) => Break(Err(e)),
            Pending => Continue(Err(Pending)),
        }
    }

    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self {
        match ctrl {
            Continue(Ok(x)) => Ready(Ok(x)),
            Break(Err(e)) => Ready(Err(e)),
            Continue(Err(Pending)) => Pending,
        }
    }
}

impl<E> TryKind for Poll<Option<Result<(), E>>> {
    type WithContinue<C> = Poll<Option<Result<C, E>>>;
    type Residual = Option<Result<Never, E>>;
    type Empty = Poll<Never>;
}

impl<T, E> CustomTry for Poll<Option<Result<T, E>>> {
    type Kind = Poll<Option<Result<(), E>>>;
    type Continue = T;

    fn into_ctrl(self) -> ControlFlowOf<Self> {
        match self {
            Ready(Some(Ok(x))) => Continue(Ok(x)),
            Ready(None) => Break(None),
            Ready(Some(Err(e))) => Break(Some(Err(e))),
            Pending => Continue(Err(Pending)),
        }
    }

    fn from_ctrl(ctrl: ControlFlowOf<Self>) -> Self {
        match ctrl {
            Continue(Ok(x)) => Ready(Some(Ok(x))),
            Break(None) => Ready(None),
            Break(Some(Err(e))) => Ready(Some(Err(e))),
            Continue(Err(Pending)) => Pending,
        }
    }
}
