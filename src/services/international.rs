//! InternationalShipping Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{BaseProductId, UserId};

use super::types::ServiceFuture;
use errors::Error;
use models::{InternationalShipping, NewInternationalShipping, UpdateInternationalShipping};
use repos::ReposFactory;

pub trait InternationalShippingService {
    /// Creates new international_shipping
    fn create(&self, payload: NewInternationalShipping) -> ServiceFuture<InternationalShipping>;

    /// Get a international_shipping
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<InternationalShipping>;

    /// Update a international_shipping
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateInternationalShipping) -> ServiceFuture<InternationalShipping>;

    /// Delete a international_shipping
    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<InternationalShipping>;
}

/// InternationalShipping services, responsible for CRUD operations
pub struct InternationalShippingServiceImpl<
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
    > InternationalShippingServiceImpl<T, M, F>
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
    > InternationalShippingService for InternationalShippingServiceImpl<T, M, F>
{
    fn create(&self, payload: NewInternationalShipping) -> ServiceFuture<InternationalShipping> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let international_shippings_repo = repo_factory.create_international_shippings_repo(&*conn, user_id);
                            international_shippings_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service InternationalShippings, create endpoint error occured.").into()),
        )
    }

    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<InternationalShipping> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let international_shippings_repo = repo_factory.create_international_shippings_repo(&*conn, user_id);
                            international_shippings_repo.get_by_base_product_id(base_product_id)
                        })
                })
                .map_err(|e| {
                    e.context("Service InternationalShippings, get_by_base_product_id endpoint error occured.")
                        .into()
                }),
        )
    }

    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateInternationalShipping) -> ServiceFuture<InternationalShipping> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let international_shippings_repo = repo_factory.create_international_shippings_repo(&*conn, user_id);
                            international_shippings_repo.update(base_product_id_arg, payload)
                        })
                })
                .map_err(|e| e.context("Service InternationalShippings, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<InternationalShipping> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let international_shippings_repo = repo_factory.create_international_shippings_repo(&*conn, user_id);
                            international_shippings_repo.delete(base_product_id_arg)
                        })
                })
                .map_err(|e| e.context("Service InternationalShippings, delete endpoint error occured.").into()),
        )
    }
}