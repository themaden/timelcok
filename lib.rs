#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec, Map};

// Veri anahtarları için enum
// Kontratın depolama alanında saklanan verilerin türlerini tanımlar
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,            // Yönetici adresi
    RewardPool,       // Ödül havuzu bilgileri
    UserRewards,      // Kullanıcılara atanan ödüller
    RewardClaimed,    // Talep edilmiş ödüller
}

// Ödül türleri için enum
// İki farklı ödül türü tanımlar
#[derive(Clone)]
#[contracttype]
pub enum RewardType {
    Fixed,            // Sabit miktar ödül (kesin rakam)
    Percentage,       // Yüzde bazlı ödül (havuzun belirli bir yüzdesi)
}

// Ödül yapısı
// Bir ödülün tüm bilgilerini saklar
#[derive(Clone)]
#[contracttype]
pub struct Reward {
    pub token: Address,         // Ödül olarak verilecek token adresi
    pub reward_type: RewardType, // Ödül türü (sabit veya yüzde)
    pub amount: i128,           // Ödül miktarı
    pub valid_until: u64,       // Ödülün geçerli olduğu son tarih (zaman damgası)
}

// Ödül havuzu yapısı
// Kontratın ödül dağıtımı için kullandığı havuz bilgilerini saklar
#[derive(Clone)]
#[contracttype]
pub struct RewardPool {
    pub token: Address,         // Havuz için kullanılan token adresi
    pub total_amount: i128,     // Havuzdaki toplam token miktarı
    pub distributed: i128,      // Şimdiye kadar dağıtılmış token miktarı
    pub active: bool,           // Havuzun aktif olup olmadığı
}

// Kontrat yapısı tanımı
#[contract]
pub struct RewardDistributionContract;

// Yardımcı fonksiyonlar
// Ödül havuzu bilgilerini getir
fn get_reward_pool(env: &Env) -> RewardPool {
    env.storage().instance().get(&DataKey::RewardPool).unwrap()
}

// Çağıranın yönetici olup olmadığını kontrol et
fn is_admin(env: &Env, caller: &Address) -> bool {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    &admin == caller
}

// Bir ödülün hala geçerli olup olmadığını kontrol et (süresi dolmamış mı)
fn check_reward_validity(env: &Env, reward: &Reward) -> bool {
    let current_timestamp = env.ledger().timestamp();
    current_timestamp <= reward.valid_until
}

// Bir ödülün daha önce talep edilip edilmediğini kontrol et
fn is_reward_claimed(env: &Env, user: &Address, reward_id: &u32) -> bool {
    let key = (user.clone(), reward_id.clone());
    env.storage().persistent().has(&DataKey::RewardClaimed, &key)
}

#[contractimpl]
impl RewardDistributionContract {
    // Kontratı başlat - yönetici, token ve başlangıç miktarı ile
    pub fn initialize(env: Env, admin: Address, token: Address, initial_amount: i128) {
        // Kontratın zaten başlatılmış olup olmadığını kontrol et
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("contract already initialized");
        }
        
        // Yönetici adresini ayarla
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        // Ödül havuzunu başlat
        let reward_pool = RewardPool {
            token,                  // Ödül tokeni
            total_amount: initial_amount, // Başlangıç miktarı
            distributed: 0,         // Henüz dağıtım yapılmadı
            active: true,           // Havuz aktif
        };
        
        // Ödül havuzunu depola
        env.storage().instance().set(&DataKey::RewardPool, &reward_pool);
        
        // Kullanıcı ödüllerini saklamak için boş harita oluştur
        let user_rewards: Map<Address, Vec<Reward>> = Map::new(&env);
        env.storage().instance().set(&DataKey::UserRewards, &user_rewards);
    }
    
    // Yöneticinin ödül havuzuna token yatırması
    pub fn deposit_to_pool(env: Env, from: Address, amount: i128) {
        // Çağıranın yönetici olup olmadığını doğrula
        if !is_admin(&env, &from) {
            panic!("only admin can deposit to pool");
        }
        
        // Yöneticinin yetkilendirmesini talep et (kimlik doğrulama)
        from.require_auth();
        
        // Mevcut ödül havuzu bilgilerini al
        let mut reward_pool = get_reward_pool(&env);
        
        // Tokenleri yöneticiden kontrata transfer et
        token::Client::new(&env, &reward_pool.token)
            .transfer(&from, &env.current_contract_address(), &amount);
        
        // Havuz miktarını güncelle
        reward_pool.total_amount += amount;
        env.storage().instance().set(&DataKey::RewardPool, &reward_pool);
    }
    
    // Yöneticinin kullanıcılara ödül ataması
    pub fn assign_reward(
        env: Env, 
        admin: Address,             // Yönetici adresi
        to: Address,                // Ödül alacak kullanıcı
        reward_type: RewardType,    // Ödül türü (Sabit/Yüzde)
        amount: i128,               // Ödül miktarı
        valid_days: u64             // Geçerlilik süresi (gün)
    ) {
        // Çağıranın yönetici olup olmadığını doğrula
        if !is_admin(&env, &admin) {
            panic!("only admin can assign rewards");
        }
        
        // Yöneticinin yetkilendirmesini talep et
        admin.require_auth();
        
        // Ödül havuzunu al ve aktif olup olmadığını kontrol et
        let reward_pool = get_reward_pool(&env);
        if !reward_pool.active {
            panic!("reward pool is not active");
        }
        
        // Geçerlilik süresini hesapla (şu anki zaman + gün * saniye)
        let current_time = env.ledger().timestamp();
        let valid_until = current_time + (valid_days * 86400); // 86400 = 1 gündeki saniye sayısı
        
        // Ödül yapısını oluştur
        let reward = Reward {
            token: reward_pool.token.clone(), // Ödül tokeni
            reward_type,                     // Ödül türü
            amount,                          // Miktar
            valid_until,                     // Geçerlilik süresi
        };
        
        // Kullanıcı ödüllerini al
        let mut user_rewards: Map<Address, Vec<Reward>> = 
            env.storage().instance().get(&DataKey::UserRewards).unwrap();
        
        // Ödülü kullanıcının listesine ekle
        if let Some(mut rewards) = user_rewards.get(to.clone()) {
            // Kullanıcının zaten ödülleri varsa listeye ekle
            rewards.push_back(reward.clone());
            user_rewards.set(to.clone(), rewards);
        } else {
            // Kullanıcının henüz ödülü yoksa yeni liste oluştur
            let mut rewards = Vec::new(&env);
            rewards.push_back(reward.clone());
            user_rewards.set(to.clone(), rewards);
        }
        
        // Kullanıcı ödülleri haritasını güncelle
        env.storage().instance().set(&DataKey::UserRewards, &user_rewards);
    }
    
    // Kullanıcının ödülünü talep etmesi
    pub fn claim_reward(env: Env, user: Address, reward_index: u32) {
        // Kullanıcının yetkilendirmesini talep et (kimlik doğrulama)
        user.require_auth();
        
        // Ödülün daha önce talep edilip edilmediğini kontrol et
        if is_reward_claimed(&env, &user, &reward_index) {
            panic!("reward already claimed");
        }
        
        // Kullanıcı ödüllerini al
        let user_rewards: Map<Address, Vec<Reward>> = 
            env.storage().instance().get(&DataKey::UserRewards).unwrap();
        
        // Kullanıcının ödüllerini al, yoksa hata ver
        let rewards = match user_rewards.get(user.clone()) {
            Some(r) => r,
            None => panic!("no rewards assigned to user"),
        };
        
        // İndeksin sınırlar içinde olup olmadığını kontrol et
        if reward_index as u32 >= rewards.len() {
            panic!("invalid reward index");
        }
        
        // Belirtilen ödülü al
        let reward = rewards.get(reward_index as u32).unwrap();
        
        // Ödülün hala geçerli olup olmadığını kontrol et
        if !check_reward_validity(&env, &reward) {
            panic!("reward has expired");
        }
        
        // Ödül havuzunu al
        let mut reward_pool = get_reward_pool(&env);
        
        // Gerçek ödül miktarını hesapla (türüne göre)
        let amount = match reward.reward_type {
            RewardType::Fixed => reward.amount, // Sabit ise doğrudan miktarı kullan
            RewardType::Percentage => {
                // Yüzde ise, havuzun belirli bir yüzdesini hesapla (1000 = %10)
                (reward_pool.total_amount * reward.amount) / 10000
            }
        };
        
        // Havuzda yeterli miktar olup olmadığını kontrol et
        if reward_pool.total_amount - reward_pool.distributed < amount {
            panic!("insufficient funds in reward pool");
        }
        
        // Ödülü kullanıcıya transfer et
        token::Client::new(&env, &reward_pool.token)
            .transfer(&env.current_contract_address(), &user, &amount);
        
        // Dağıtılan miktarı güncelle
        reward_pool.distributed += amount;
        env.storage().instance().set(&DataKey::RewardPool, &reward_pool);
        
        // Ödülü talep edildi olarak işaretle
        let key = (user.clone(), reward_index);
        env.storage().persistent().set(&DataKey::RewardClaimed, &key, &true);
    }
    
    // Yöneticinin havuz durumunu değiştirmesi
    pub fn set_pool_status(env: Env, admin: Address, active: bool) {
        // Çağıranın yönetici olup olmadığını doğrula
        if !is_admin(&env, &admin) {
            panic!("only admin can change pool status");
        }
        
        // Yöneticinin yetkilendirmesini talep et
        admin.require_auth();
        
        // Havuz durumunu güncelle (aktif/pasif)
        let mut reward_pool = get_reward_pool(&env);
        reward_pool.active = active;
        env.storage().instance().set(&DataKey::RewardPool, &reward_pool);
    }
    
    // Görüntüleme fonksiyonları
    
    // Bir kullanıcının tüm ödüllerini görüntüle
    pub fn get_user_rewards(env: Env, user: Address) -> Vec<Reward> {
        let user_rewards: Map<Address, Vec<Reward>> = 
            env.storage().instance().get(&DataKey::UserRewards).unwrap();
            
        match user_rewards.get(user) {
            Some(rewards) => rewards,  // Kullanıcının ödülleri varsa döndür
            None => Vec::new(&env),    // Yoksa boş liste döndür
        }
    }
    
    // Ödül havuzu bilgilerini görüntüle
    pub fn get_pool_info(env: Env) -> RewardPool {
        get_reward_pool(&env)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, vec, map};
    
    #[test]
    fn test_reward_distribution() {
        // Test modülü - kontratın doğru çalıştığını doğrulamak için testler
        
        // Test ortamını, yönetici, kullanıcı ve token adresi oluştur
        let env = Env::default();
        let admin = Address::random(&env);
        let user = Address::random(&env);
        let token = Address::random(&env);
        
        // Kontratı başlat - yönetici, token ve 10000 başlangıç miktarı ile
        RewardDistributionContract::initialize(&env, admin.clone(), token.clone(), 10000);
        
        // Yönetici kullanıcıya ödül atar:
        // - Sabit miktar ödül (100 birim)
        // - 30 gün geçerlilik süresi
        RewardDistributionContract::assign_reward(
            &env, 
            admin.clone(), 
            user.clone(), 
            RewardType::Fixed, 
            100, 
            30
        );
        
        // Kullanıcı ödüllerini kontrol et - kullanıcının bir ödülü olmalı
        let rewards = RewardDistributionContract::get_user_rewards(&env, user.clone());
        assert_eq!(rewards.len(), 1);
        
        // Ödül talep etme testi
        // (Gerçek testte, token transferlerini simüle etmek gerekir)
        // Not: Bu test tamamlanmamıştır, gerçek bir uygulamada daha kapsamlı testler yazılmalıdır
    }
}