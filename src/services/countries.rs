//! Countries Services, presents CRUD operations with countries

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::UserId;

use errors::Error;

use super::types::ServiceFuture;
use models::{Country, NewCountry};
use repos::ReposFactory;

pub trait CountriesService {
    /// Returns country by label
    fn get(&self, label: String) -> ServiceFuture<Option<Country>>;
    /// Creates new country
    fn create(&self, payload: NewCountry) -> ServiceFuture<Country>;
    /// Returns all countries as a tree
    fn get_all(&self) -> ServiceFuture<Country>;
}

/// Countries services, responsible for Country-related CRUD operations
pub struct CountriesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CountriesServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<UserId>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CountriesService for CountriesServiceImpl<T, M, F>
{
    /// Returns country by label
    fn get(&self, label: String) -> ServiceFuture<Option<Country>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                            countries_repo.find(label)
                        })
                })
                .map_err(|e| e.context("Service Countries, get endpoint error occured.").into()),
        )
    }

    /// Creates new country
    fn create(&self, new_country: NewCountry) -> ServiceFuture<Country> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                            conn.transaction::<(Country), FailureError, _>(move || countries_repo.create(new_country))
                        })
                })
                .map_err(|e| e.context("Service Countries, create endpoint error occured.").into()),
        )
    }

    /// Returns country by label
    fn get_all(&self) -> ServiceFuture<Country> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                            countries_repo.get_all()
                        })
                })
                .map_err(|e| e.context("Service Countries, get_all endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use futures_cpupool::CpuPool;
    use r2d2;
    use serde_json;
    use tokio_core::reactor::Core;

    use stq_types::*;

    use super::*;
    use models::*;
    use repos::repo_factory::tests::*;

    fn create_countries_service(user_id: Option<UserId>) -> CountriesServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        CountriesServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_countries(label: &str) -> NewCountry {
        NewCountry {
            label: label.to_string(),
            name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
            level: 3,
            parent_label: Some("EEE".to_string()),
        }
    }

    #[test]
    fn test_get_countries() {
        let mut core = Core::new().unwrap();
        let service = create_countries_service(Some(MOCK_USER_ID));
        let work = service.get("root".to_string());
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().label, "root".to_string());
    }

    #[test]
    fn test_create_countries() {
        let mut core = Core::new().unwrap();
        let service = create_countries_service(Some(MOCK_USER_ID));
        let new_countries = create_new_countries("rus");
        let work = service.create(new_countries);
        let result = core.run(work).unwrap();
        assert_eq!(result.label, "rus".to_string());
    }

}