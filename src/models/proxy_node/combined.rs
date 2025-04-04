use super::vless::VlessProxy;

#[derive(Debug, Clone)]
pub enum CombinedProxy {
    Vless(VlessProxy),
}
