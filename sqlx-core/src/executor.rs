use futures_core::future::BoxFuture;

use crate::database::{Database, HasCursor};
use crate::describe::Describe;

/// A type that contains or can provide a database connection to use for executing queries
/// against the database.
///
/// No guarantees are provided that successive queries run on the same physical database
/// connection. A [`Connection`](trait.Connection.html) is an `Executor` that guarantees that successive
/// queries are run on the same physical database connection.
///
/// Implementations are provided for [`&Pool`](struct.Pool.html),
/// [`&mut PoolConnection`](struct.PoolConnection.html),
/// and [`&mut Connection`](trait.Connection.html).
pub trait Executor
where
    Self: Send,
{
    /// The specific database that this type is implemented for.
    type Database: Database;

    /// Executes the query for its side-effects and
    /// discarding any potential result rows.
    ///
    /// Returns the number of rows affected, or 0 if not applicable.
    fn execute<'e, 'q, E: 'e>(&'e mut self, query: E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>;

    fn fetch<'e, 'q, E>(&'e mut self, query: E) -> <Self::Database as HasCursor<'e, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>;

    /// Prepare the SQL query and return type information about its parameters
    /// and results.
    ///
    /// This is used by the query macros ( [`query!`] ) during compilation to
    /// power their type inference.
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>;
}

pub trait RefExecutor<'c> {
    type Database: Database;

    /// Executes a query for its result.
    ///
    /// Returns a [`Cursor`] that can be used to iterate through the [`Row`]s
    /// of the result.
    fn fetch_by_ref<'q, E>(self, query: E) -> <Self::Database as HasCursor<'c, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>;
}

/// A type that may be executed against a database connection.
pub trait Execute<'q, DB>
where
    Self: Send,
    DB: Database,
{
    /// Returns the query to be executed and the arguments to bind against the query, if any.
    ///
    /// Returning `None` for `Arguments` indicates to use a "simple" query protocol and to not
    /// prepare the query. Returning `Some(Default::default())` is an empty arguments object that
    /// will be prepared (and cached) before execution.
    fn into_parts(self) -> (&'q str, Option<DB::Arguments>);
}

impl<'q, DB> Execute<'q, DB> for &'q str
where
    DB: Database,
{
    #[inline]
    fn into_parts(self) -> (&'q str, Option<DB::Arguments>) {
        (self, None)
    }
}

macro_rules! impl_execute_for_query {
    ($db:ty) => {
        impl<'q> $crate::executor::Execute<'q, $db> for $crate::query::Query<'q, $db> {
            fn into_parts(
                self,
            ) -> (
                &'q str,
                Option<<$db as $crate::database::Database>::Arguments>,
            ) {
                (self.query, Some(self.arguments))
            }
        }
    };
}

impl<T> Executor for &'_ mut T
where
    T: Executor,
{
    type Database = T::Database;

    fn execute<'e, 'q, E: 'e>(&'e mut self, query: E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).execute(query)
    }

    fn fetch<'e, 'q, E>(&'e mut self, query: E) -> <Self::Database as HasCursor<'_, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).fetch(query)
    }

    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).describe(query)
    }
}
