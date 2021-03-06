pub mod acl;
pub mod companies;
pub mod companies_packages;
pub mod countries;
pub mod packages;
pub mod pickups;
pub mod products;
pub mod repo_factory;
pub mod shipping_rates;
pub mod types;
pub mod user_addresses;
pub mod user_roles;

pub use self::acl::*;
pub use self::companies::*;
pub use self::companies_packages::*;
pub use self::countries::*;
pub use self::packages::*;
pub use self::pickups::*;
pub use self::products::*;
pub use self::repo_factory::*;
pub use self::shipping_rates::*;
pub use self::types::*;
pub use self::user_addresses::*;
pub use self::user_roles::*;

use stq_types::Alpha3;

pub fn get_pg_str_json_array(countries: Vec<Alpha3>) -> String {
    let res = countries
        .into_iter()
        .map(|s| format!("'{}'", s.0))
        .collect::<Vec<String>>()
        .join(",");
    format!("array[{}]", res)
}

pub fn get_company_package_name(company_name: &str, package_name: &str) -> String {
    format!("{}-{}", company_name, package_name)
}
