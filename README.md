# Soroban Ödül Dağıtım Sistemi

Bu proje, Stellar blokzincirinin akıllı kontrat platformu Soroban üzerinde çalışan bir ödül dağıtım sistemi kontratıdır. Bu kontrat, organizasyonların token bazlı ödülleri belirli kullanıcılara dağıtmasını ve bu ödüllerin belirli zaman şartları altında talep edilmesini sağlar.

## Özellikler

- **Esnek Ödül Yapısı**: Sabit miktar veya yüzde bazlı ödüller tanımlayabilme
- **Zaman Kısıtlamaları**: Ödüller için son geçerlilik tarihi belirleyebilme
- **Çoklu Alıcı Desteği**: Birden fazla adrese ödül atayabilme
- **Güvenlik Kontrolleri**: Yetkisiz taleplere karşı koruma
- **Yönetici Kontrolleri**: Ödül havuzu durumunu değiştirebilme (aktif/pasif)

## Kullanım Senaryoları

- **Çalışan Ödülleri**: Performans bazlı ödül dağıtımı
- **Topluluk Teşvikleri**: Ekosistem katkıları için token ödülleri
- **Sadakat Programları**: Kullanıcı sadakat puanlarının token olarak dağıtımı
- **Vesting Planları**: Zamana yayılmış token dağıtımları

## Teknik Mimari

Kontrat, birkaç temel bileşenden oluşur:

1. **DataKey**: Kontratın depolama yapısını tanımlar
2. **RewardType**: Ödül türlerini (Sabit/Yüzde) tanımlar
3. **Reward**: Ödül bilgilerini saklayan yapı
4. **RewardPool**: Ödül havuzu bilgilerini saklayan yapı
5. **RewardDistributionContract**: Ana kontrat uygulaması

## Fonksiyonlar

### Yönetici Fonksiyonları

- `initialize`: Kontratı başlatır ve yöneticiyi belirler
- `deposit_to_pool`: Havuza token ekler
- `assign_reward`: Kullanıcılara ödül atar
- `set_pool_status`: Havuzun aktif/pasif durumunu değiştirir

### Kullanıcı Fonksiyonları

- `claim_reward`: Kullanıcının kendisine atanan ödülü talep etmesi
- `get_user_rewards`: Kullanıcının ödüllerini görüntülemesi
- `get_pool_info`: Havuz bilgilerini görüntüleme

## Kurulum

### Gereksinimler

- Rust
- Cargo
- Soroban CLI

### Adımlar

1. Projeyi klonlayın:
   ```
   git clone https://github.com/kullaniciadi/reward-distribution.git
   cd reward-distribution
   ```

2. Bağımlılıkları kurun:
   ```
   cargo build
   ```

3. Testi çalıştırın:
   ```
   cargo test
   ```

4. Kontratı derleyin:
   ```
   cargo build --target wasm32-unknown-unknown --release
   ```

5. WASM dosyasını Soroban'a yükleyin:
   ```
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/reward_distribution.wasm
   ```

## Güvenlik Notları

- Kontrat, yetkilendirme için `require_auth()` kullanır
- Kullanıcılar yalnızca kendilerine atanan ödülleri talep edebilir
- Ödüller yalnızca belirtilen zaman diliminde talep edilebilir
- Her ödül yalnızca bir kez talep edilebilir

## Lisans

Bu proje [MIT Lisansı](LICENSE) altında lisanslanmıştır.

## Katkı

Katkılarınızı bekliyoruz! Lütfen bir pull request açın veya bir sorun bildirin.

---

Bu kontrat, eğitim amaçlıdır ve üretim ortamında kullanılmadan önce kapsamlı bir denetimden geçirilmelidir.
