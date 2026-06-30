/// Result of racing two futures.
/// `Left` means the first future completed first; `Right` means the second
/// completed first.
#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
