//! REPO Products table. Products is an entity that
//! contains info about international shipping of base_product.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{BaseProductId, CompanyPackageId, UserId};

use models::authorization::*;
use models::{CompaniesPackages, NewProducts, NewProductsRaw, PackagesRaw, Products, ProductsRaw, UpdateProducts, UserRole};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;
use schema::companies_packages::dsl as DslCompaniesPackages;
use schema::packages::dsl as DslPackages;
use schema::products::dsl as DslProducts;
use schema::roles::dsl as Roles;

pub struct ProductsWithAvailableCountries(pub Products, pub Vec<CountryLabel>);

/// Products repository for handling Products
pub trait ProductsRepo {
    /// Create a new products
    fn create(&self, payload: NewProducts) -> RepoResult<Products>;

    /// Create a new products
    fn create_many(&self, payload: Vec<NewProducts>) -> RepoResult<Vec<Products>>;

    /// Get a products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Vec<Products>>;

    /// Get a products with available countries for delivery by package
    fn get_products_countries(&self, base_product_id: BaseProductId) -> RepoResult<Vec<ProductsWithAvailableCountries>>;

    /// Update a products
    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> RepoResult<Products>;

    /// Delete a products
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>>;
}

pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepo for ProductsRepoImpl<'a, T> {
    fn create(&self, payload: NewProducts) -> RepoResult<Products> {
        debug!("create new products {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(DslProducts::products).values(&payload);
        query
            .get_result::<ProductsRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|products_| products_.to_products())
            .and_then(|product| {
                acl::check(&*self.acl, Resource::Products, Action::Create, self, Some(&product))?;
                Ok(product)
            })
            .map_err(|e: FailureError| e.context(format!("create new products {:?}.", payload)).into())
    }

    fn create_many(&self, payload: Vec<NewProducts>) -> RepoResult<Vec<Products>> {
        debug!("create many new products {:?}.", payload);
        let payload = payload
            .into_iter()
            .map(|v| v.to_raw().map_err(From::from))
            .collect::<RepoResult<Vec<NewProductsRaw>>>()?;

        let query = diesel::insert_into(DslProducts::products).values(&payload);
        query
            .get_results::<ProductsRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut new_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Create, self, Some(&product))?;
                    new_products.push(product);
                }

                new_products.sort_by(|a, b| a.id.cmp(&b.id));

                Ok(new_products)
            })
            .map_err(|e: FailureError| e.context(format!("create many new products {:?}.", payload)).into())
    }

    fn get_by_base_product_id(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
        debug!("get products by base_product_id {:?}.", base_product_id_arg);
        let query = DslProducts::products.filter(DslProducts::base_product_id.eq(base_product_id_arg));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut new_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                    new_products.push(product);
                }
                Ok(new_products)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Getting products with base_product_id {:?} failed.", base_product_id_arg))
                    .into()
            })
    }

    /// Get a products with countries from packages
    fn get_products_countries(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<ProductsWithAvailableCountries>> {
        debug!(
            "Find in available countries for delivery by base_product_id: {:?}.",
            base_product_id_arg
        );

        let query = DslProducts::products
            .filter(DslProducts::base_product_id.eq(base_product_id_arg))
            .inner_join(DslCompaniesPackages::companies_packages.inner_join(DslPackages::packages))
            .order(DslPackages::id);

        query
            .get_results::<(ProductsRaw, (CompaniesPackages, PackagesRaw))>(self.db_conn)
            .map_err(From::from)
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (product_raw, (_, package_raw)) = result;
                    let element = ProductsWithAvailableCountries(product_raw.to_products()?, package_raw.to_packages()?.deliveries_to);

                    data.push(element);
                }
                Ok(data)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in available countries for delivery by base_product_id: {:?} error occured",
                    base_product_id_arg
                )).into()
            })
    }

    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id_arg: CompanyPackageId,
        payload: UpdateProducts,
    ) -> RepoResult<Products> {
        debug!("Updating products payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(
            DslProducts::products
                .filter(DslProducts::base_product_id.eq(base_product_id_arg))
                .filter(DslProducts::company_package_id.eq(company_package_id_arg)),
        ).and_then(|products_: ProductsRaw| products_.to_products())
            .and_then(|product: Products| acl::check(&*self.acl, Resource::Products, Action::Update, self, Some(&product)))
            .and_then(|_| {
                let filter = DslProducts::products
                    .filter(DslProducts::base_product_id.eq(base_product_id_arg))
                    .filter(DslProducts::company_package_id.eq(company_package_id_arg));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<ProductsRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(|products_| products_.to_products())
            .map_err(|e: FailureError| e.context(format!("Updating products payload {:?} failed.", payload)).into())
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
        debug!("delete products {:?}.", base_product_id_arg);
        let query = DslProducts::products.filter(DslProducts::base_product_id.eq(base_product_id_arg));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut delete_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Delete, self, Some(&product))?;
                    delete_products.push(product);
                }
                Ok(delete_products)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Delete products with base product id {:?} failed.", base_product_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Products>
    for ProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&Products>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(obj) = obj {
                    Roles::roles
                        .filter(Roles::user_id.eq(user_id_arg))
                        .get_results::<UserRole>(self.db_conn)
                        .map_err(From::from)
                        .map(|user_roles_arg| {
                            user_roles_arg
                                .iter()
                                .any(|user_role_arg| user_role_arg.data.clone().map(|data| data == obj.store_id.0).unwrap_or_default())
                        })
                        .unwrap_or_else(|_: FailureError| false)
                } else {
                    false
                }
            }
        }
    }
}
