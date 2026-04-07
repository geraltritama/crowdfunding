# 📦 Dokumentasi Superteam Indonesia — Weekend Class
### Platform Crowdfunding Solana + Kriteria Code Review

---

# BAGIAN 1: Project Brief — Platform Crowdfunding di Solana

## Apa yang Kita Bangun

Smart contract crowdfunding di Solana di mana pengguna dapat membuat kampanye, menerima donasi, dan bisa mengklaim dana (jika berhasil) atau mengembalikan dana (jika gagal).

> **Bayangkan Kickstarter, tapi di atas blockchain.**

---

## Masalah yang Diselesaikan

Saat ini, tidak ada cara bagi para kreator untuk:

- Menerima donasi tanpa langsung menerimanya
- Menjamin pengembalian dana jika target tidak tercapai
- Membuktikan kepada donatur bahwa dana dikunci hingga kondisi terpenuhi

---

## Tugasmu

Bangun sebuah program Solana dengan **4 fungsi**:

---

### 1. Buat Kampanye *(Create Campaign)*

**Fungsi:** Kreator membuat kampanye penggalangan dana baru

**Input:**
- `goal` (u64): Target jumlah dalam lamports
- `deadline` (i64): Unix timestamp saat kampanye berakhir

**Logika:**
- Simpan data kampanye (creator, goal, deadline, raised=0, claimed=false)
- Validasi bahwa deadline ada di masa depan
- Log: `"Campaign created: goal={goal}, deadline={deadline}"`

---

### 2. Kontribusi *(Contribute)*

**Fungsi:** Donatur mengirim SOL ke kampanye

**Input:**
- `amount` (u64): Jumlah donasi dalam lamports

**Logika:**
- Transfer SOL dari donatur ke vault kampanye (PDA)
- Perbarui `campaign.raised += amount`
- Log: `"Contributed: {amount} lamports, total={raised}"`

---

### 3. Penarikan Dana *(Withdraw)*

**Fungsi:** Kreator mengklaim dana jika kampanye berhasil

**Syarat (semua harus terpenuhi):**
- Jumlah terkumpul >= goal
- Waktu saat ini >= deadline
- Pemanggil adalah kreator kampanye
- Kampanye belum diklaim sebelumnya

**Logika:**
- Transfer semua SOL dari vault ke kreator
- Tandai `campaign.claimed = true`
- Log: `"Withdrawn: {amount} lamports"`

---

### 4. Pengembalian Dana *(Refund)*

**Fungsi:** Donatur mendapatkan uangnya kembali jika kampanye gagal

**Syarat (semua harus terpenuhi):**
- Jumlah terkumpul < goal
- Waktu saat ini >= deadline

**Logika:**
- Transfer kontribusi donatur kembali dari vault
- Log: `"Refunded: {amount} lamports"`

---

## Spesifikasi Teknis

### Struktur Data

```rust
pub struct Campaign {
    pub creator: Pubkey,    // Siapa yang membuat kampanye ini
    pub goal: u64,          // Target jumlah
    pub raised: u64,        // Jumlah yang sudah terkumpul
    pub deadline: i64,      // Kapan kampanye berakhir
    pub claimed: bool,      // Sudah ditarik?
}
```

---

### Vault (Penting!)

Jangan kirim donasi langsung ke kreator. Gunakan **Program Derived Address (PDA)** sebagai vault:

```rust
// Derive alamat vault
let (vault_pda, bump) = Pubkey::find_program_address(
    &[b"vault", campaign_account.key.as_ref()],
    program_id
);

// Saat transfer DARI vault, gunakan invoke_signed:
invoke_signed(
    &system_instruction::transfer(vault_pda, recipient, amount),
    &[vault_account, recipient_account, system_program],
    &[&[b"vault", campaign_account.key.as_ref(), &[bump]]]
)?;
```

> **Kenapa PDA?** Ini adalah akun yang dikontrol oleh programmu. Tidak perlu private key — programmu bisa "menandatangani" transaksi untuknya.

---

### Mendapatkan Waktu Saat Ini

```rust
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

let clock = Clock::get()?;
let current_time = clock.unix_timestamp;
```

---

## Kriteria Keberhasilan

Programmu harus:

- ✅ Menerima pembuatan kampanye dengan goal dan deadline
- ✅ Menerima kontribusi dan melacak total yang terkumpul
- ✅ Mengizinkan penarikan hanya jika goal tercapai setelah deadline
- ✅ Mengizinkan refund hanya jika goal TIDAK tercapai setelah deadline
- ✅ Mencegah penarikan ganda
- ✅ Menggunakan PDA untuk vault (bukan transfer langsung)

---

## Checklist Pengujian

1. Buat kampanye dengan goal=1000 SOL, deadline=besok
2. Kontribusi 600 SOL → harus berhasil, raised=600
3. Kontribusi 500 SOL → harus berhasil, raised=1100
4. Coba tarik sebelum deadline → harus **gagal**
5. Tunggu hingga setelah deadline → penarikan harus **berhasil**
6. Coba tarik lagi → harus **gagal** (sudah diklaim)

---

## Kesalahan Umum

| ❌ Jangan | ✅ Lakukan |
|---|---|
| Kirim donasi langsung ke kreator | Gunakan vault PDA |
| Izinkan penarikan sebelum deadline | Cek goal DAN waktu |
| Lupa tandai claimed=true | Cegah penarikan ganda |
| Pakai `unwrap()` di mana-mana | Tangani error dengan benar |

---

## Sumber Daya

- [Solana Cookbook - PDAs](https://solanacookbook.com)
- [Solana Cookbook - CPIs](https://solanacookbook.com)

---

## Deliverables

1. Kode program Rust
2. Deploy ke Solana Devnet
3. Program ID
4. Tanda tangan transaksi pengujian

**Semangat!** 🚀

---
---

# BAGIAN 2: Code Review — Kriteria Penilaian

### Kriteria komprehensif untuk mengevaluasi submission kode blockchain di ekosistem EVM (Solidity) dan Solana (Rust/Anchor).

---

## Gambaran Penilaian

**Total Skor: 100 poin dalam 6 kategori**

| Kategori | Poin Maks | Bobot |
|---|---|---|
| Fungsionalitas | 30 | 30% |
| Kualitas Kode | 20 | 20% |
| Desain | 20 | 20% |
| Dokumentasi | 15 | 15% |
| Keamanan | 10 | 10% |
| Inovasi | 5 | 5% |

### Indikator Status

- 🟢 **Baik (Hijau):** >80% dari poin maksimum kategori
- 🟡 **Peringatan (Kuning):** 50–80% dari poin maksimum kategori
- 🔴 **Kritis (Merah):** <50% dari poin maksimum kategori

---

## 1. Fungsionalitas (30 poin)

**Pertanyaan Inti: Apakah kode memenuhi persyaratan tantangan?**

### Yang Kami Tinjau:

**Kelengkapan Persyaratan**
- Semua fitur yang ditentukan sudah diimplementasikan
- Fungsionalitas inti bekerja sesuai deskripsi di brief tantangan
- Kasus tepi ditangani dengan tepat
- Validasi input tersedia

**Kebenaran Logika Bisnis**

*EVM/Solidity:*
- Transisi state mengikuti alur yang diharapkan
- Event dipancarkan pada titik yang tepat
- Modifier fungsi digunakan dengan benar
- Nilai kembalian sesuai spesifikasi

*Solana/Anchor:*
- Handler instruksi mengimplementasikan operasi yang diperlukan
- State akun diperbarui dengan benar
- Cross-program invocations (CPI) berjalan sesuai tujuan
- Program derived addresses (PDAs) digenerate dengan benar

**Integrasi & Interoperabilitas**
- Pemanggilan kontrak/program eksternal berfungsi dengan baik
- Standar token diikuti (ERC-20, ERC-721, SPL Token)
- Integrasi oracle bekerja dengan benar
- Fitur multi-signature atau governance beroperasi dengan baik

**Bukti Pengujian**
- Fungsionalitas inti memiliki cakupan pengujian
- Pengujian mendemonstrasikan pemenuhan persyaratan
- Integration test ada untuk alur yang kompleks

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 27–30 | Semua persyaratan terpenuhi, kasus tepi ditangani, diuji dengan baik |
| 21–26 | Persyaratan inti terpenuhi, celah kecil pada kasus tepi |
| 15–20 | Sebagian besar persyaratan terpenuhi, beberapa fungsionalitas tidak lengkap |
| <15 | Fungsionalitas utama hilang atau rusak |

---

## 2. Kualitas Kode (20 poin)

**Pertanyaan Inti: Apakah kode bersih, mudah dibaca, dan mudah dipelihara?**

### Yang Kami Tinjau:

**Organisasi Kode**
- Struktur file yang logis
- Pemisahan kepentingan yang jelas
- Desain modular dengan komponen yang dapat digunakan kembali
- Penggunaan library vs kode kustom yang tepat

**Konvensi Penamaan**

*EVM/Solidity:*
- Contract: `PascalCase`
- Fungsi: `camelCase`
- Konstanta: `UPPER_SNAKE_CASE`
- Fungsi private: diawali underscore

*Solana/Anchor:*
- Struct: `PascalCase`
- Fungsi: `snake_case`
- Konstanta: `UPPER_SNAKE_CASE`
- Modul diorganisir secara logis

**Kejelasan Kode**
- Fungsi memiliki tanggung jawab tunggal
- Duplikasi kode minimal (prinsip DRY)
- Logika kompleks dipecah menjadi fungsi-fungsi lebih kecil
- Magic number diganti dengan konstanta bernama
- Penggunaan komentar yang tepat untuk logika kompleks

**Penanganan Error**

*EVM/Solidity:*
- Custom error lebih diutamakan daripada require strings (efisiensi gas)
- Pesan error yang bermakna
- Penggunaan `revert`, `require`, `assert` yang tepat

*Solana/Anchor:*
- Custom error enum didefinisikan
- Pesan error yang deskriptif
- Penanganan `Result<T>` yang tepat

**Efisiensi Kode**
- Tidak ada komputasi yang tidak perlu
- Struktur data yang efisien dipilih
- Loop dioptimalkan atau dihindari jika memungkinkan
- EVM: Optimasi gas dipertimbangkan
- Solana: Optimasi compute unit dipertimbangkan

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 18–20 | Kualitas kode teladan, mengikuti semua best practice |
| 14–17 | Kualitas baik, inkonsistensi gaya minor |
| 10–13 | Dapat diterima tapi perlu refactoring di beberapa bagian |
| <10 | Organisasi buruk, sulit dibaca/dipelihara |

---

## 3. Desain (20 poin)

**Pertanyaan Inti: Apakah arsitekturnya solid dan mengikuti pola terbaik?**

### Yang Kami Tinjau:

**Pola Arsitektur**

*EVM/Solidity:*
- Penggunaan inheritance vs composition yang tepat
- Proxy pattern untuk upgradeability (jika diperlukan)
- Factory pattern untuk deployment kontrak
- Pola akses kontrol (Ownable, AccessControl)
- State machine pattern di mana berlaku

*Solana/Anchor:*
- Desain struktur akun
- Strategi derivasi PDA
- Organisasi handler instruksi
- Pendekatan manajemen state
- Arsitektur validasi akun

**Struktur Data**
- Layout penyimpanan yang efisien
- EVM: Penggunaan storage vs memory vs calldata
- Solana: Optimasi ukuran akun
- Penggunaan mapping/array vs struct yang tepat
- Packing data di mana bermanfaat

**Pertimbangan Skalabilitas**
- Desain mendukung ekstensi di masa depan
- Cukup modular untuk menambahkan fitur
- EVM: Upgradeable jika persyaratan menyarankannya
- Solana: Reallokasi akun dipertimbangkan
- Operasi batch didukung di mana diperlukan

**Pola Desain**

*EVM:*
- Pola Checks-Effects-Interactions
- Pull over Push untuk pembayaran
- Fungsionalitas circuit breaker/pause
- Rate limiting di mana sesuai

*Solana:*
- Anchor constraints digunakan secara efektif
- Pola validasi akun
- Pola otorisasi signer
- Pola penanganan token account

**Desain Interface**
- API yang bersih dan intuitif
- Tanda tangan fungsi yang konsisten
- Modifier visibilitas yang tepat
- Event/log untuk perubahan state penting

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 18–20 | Arsitektur sangat baik, pola standar industri |
| 14–17 | Desain solid, perbaikan minor dimungkinkan |
| 10–13 | Fungsional tapi pilihan desain kurang optimal |
| <10 | Arsitektur buruk, cacat desain mayor |

---

## 4. Dokumentasi (15 poin)

**Pertanyaan Inti: Apakah kode terdokumentasi dengan baik untuk para developer?**

### Yang Kami Tinjau:

**Kualitas README**
- Deskripsi proyek yang jelas
- Instruksi setup lengkap
- Dependensi tercantum
- Langkah build/deployment
- Contoh penggunaan tersedia
- Gambaran arsitektur

**Komentar Kode**

*Dokumentasi Fungsi:*
- Tujuan dinyatakan dengan jelas
- Parameter dijelaskan
- Nilai kembalian didokumentasikan
- Efek samping dicatat
- EVM: Format NatSpec (`@notice`, `@dev`, `@param`, `@return`)
- Solana: Rust doc comments (`///`)

*Komentar Inline:*
- Logika kompleks dijelaskan
- "Kenapa" bukan hanya "apa"
- Pertimbangan keamanan dicatat
- TODO ditandai dengan tepat

**Dokumentasi Teknis**
- Diagram arsitektur (jika kompleks)
- Diagram alur state
- Panduan integrasi
- Dokumentasi API
- EVM: Dokumentasi ABI
- Solana: IDL digenerate dengan benar

**Contoh & Panduan**
- Contoh penggunaan dalam pengujian atau skrip
- Skenario umum didokumentasikan
- Contoh penanganan error
- Contoh integrasi

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 13–15 | Dokumentasi komprehensif, mudah dipahami |
| 10–12 | Dokumentasi baik, celah minor |
| 7–9 | Dokumentasi dasar, perlu lebih detail |
| <7 | Dokumentasi buruk atau tidak ada |

---

## 5. Keamanan (10 poin)

**Pertanyaan Inti: Apakah kode aman dan mengikuti best practice?**

### Yang Kami Tinjau:

**Kerentanan Umum**

*EVM/Solidity:*
- ✓ Perlindungan reentrancy (ReentrancyGuard, pola CEI)
- ✓ Integer overflow/underflow (Solidity 0.8+ atau SafeMath)
- ✓ Akses kontrol diimplementasikan dengan benar
- ✓ Pertimbangan front-running
- ✓ Ketergantungan pada timestamp dihindari
- ✓ Delegatecall digunakan dengan aman
- ✓ `tx.origin` tidak digunakan untuk otorisasi
- ✓ Keacakan yang tepat (bukan block.timestamp/blockhash)
- ✓ Pertimbangan serangan flash loan
- ✓ Perlindungan manipulasi oracle harga

*Solana/Anchor:*
- ✓ Pemeriksaan signer pada semua operasi yang membutuhkan hak istimewa
- ✓ Validasi kepemilikan akun
- ✓ Validasi PDA (seeds sesuai yang diharapkan)
- ✓ Validasi data akun
- ✓ Pemeriksaan overflow aritmatika
- ✓ Penggunaan Anchor constraints yang tepat (`#[account(...)]`)
- ✓ Kerentanan penutupan akun dicegah
- ✓ Serangan reinisialisasi dicegah
- ✓ Serangan kebingungan tipe dicegah
- ✓ Akun mutable duplikat ditangani

**Validasi Input**
- Semua input pengguna divalidasi
- Kondisi batas diperiksa
- Pemeriksaan zero address/account
- Validasi panjang array
- Pemeriksaan jumlah/saldo sebelum operasi

**Akses Kontrol**
- Fungsi memiliki visibilitas yang tepat
- Akses berbasis peran diimplementasikan dengan benar
- Persyaratan multi-sig di mana diperlukan
- Fungsi admin dilindungi
- Mekanisme transfer kepemilikan aman

**Keamanan Aset**

*EVM:*
- Penanganan Ether aman
- Transfer token menggunakan SafeERC20
- Race condition approval ditangani
- Pemeriksaan saldo sebelum transfer

*Solana:*
- Validasi token account
- Pemeriksaan otoritas pada operasi token
- Penggunaan token program yang tepat
- Rent exemption ditangani

**Best Practice**
- Pemanggilan eksternal ditangani dengan aman
- Mekanisme fail-safe tersedia
- Fungsionalitas emergency stop (jika sesuai)
- Rate limiting pada operasi sensitif
- Emisi event yang tepat untuk pemantauan

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 9–10 | Tidak ada masalah keamanan, mengikuti semua best practice |
| 7–8 | Perbaikan keamanan minor diperlukan |
| 5–6 | Beberapa masalah keamanan ada |
| <5 | Kerentanan keamanan kritis ditemukan |

---

## 6. Inovasi (5 poin)

**Pertanyaan Inti: Apakah kode menunjukkan kreativitas dan pendekatan unik?**

### Yang Kami Tinjau:

**Solusi Kreatif**
- Pendekatan baru untuk masalah umum
- Penggunaan primitif blockchain yang cerdas
- Pola desain yang inovatif
- Implementasi fitur yang unik

**Keunggulan Teknis**
- Teknik lanjutan digunakan dengan tepat
- Optimasi melampaui pendekatan standar
- Penggunaan fitur bahasa yang kreatif
- Solusi elegan untuk masalah kompleks

**Pengalaman Pengguna**
- Pertimbangan UX yang matang
- Optimasi gas/compute untuk pengguna
- Interface yang intuitif
- Pesan error yang membantu pengguna

**Melampaui Persyaratan**
- Fitur tambahan yang berguna
- Perhatian ekstra pada detail dan polish
- Cakupan pengujian yang komprehensif
- Pertimbangan siap produksi

### Panduan Penilaian:

| Skor | Keterangan |
|---|---|
| 5 | Kreativitas dan inovasi luar biasa |
| 4 | Elemen inovatif yang menonjol |
| 3 | Implementasi standar, dieksekusi dengan baik |
| 2 | Implementasi dasar, tidak ada fitur unggulan |
| 1 | Usaha minimal, hanya memenuhi persyaratan dasar |

---

## Proses Review

**1. Validasi Awal**
- Kode sesuai persyaratan tantangan
- Repository dapat diakses dan lengkap
- Bahasa utama sesuai spesifikasi tantangan

**2. Analisis Otomatis**
- Pemeriksaan struktur file
- Metrik ukuran dan kompleksitas kode
- Analisis dependensi

**3. Review Mendalam**
- Analisis kode baris per baris
- Pencocokan pola untuk kerentanan
- Evaluasi arsitektur
- Penilaian dokumentasi

**4. Penilaian & Umpan Balik**
- Skor kategori ditetapkan
- Masalah kritis ditandai
- Peringatan untuk perbaikan
- Kekuatan disorot
- Sumber daya pembelajaran disediakan

---

## Tingkat Keparahan Masalah

### 🔴 Masalah Kritis
- Kerentanan keamanan
- Fungsionalitas inti yang rusak
- Risiko kehilangan data
- Bug yang dapat dieksploitasi

> **Tindakan Wajib:** Harus diperbaiki sebelum produksi

### 🟡 Peringatan
- Masalah kualitas kode
- Inefisiensi gas/compute
- Penanganan kasus tepi yang hilang
- Celah dokumentasi
- Inkonsistensi gaya

> **Tindakan Disarankan:** Sebaiknya diperbaiki untuk kualitas siap produksi

### 🟢 Kekuatan
- Implementasi yang sangat baik
- Contoh best practice
- Solusi inovatif
- Area kualitas luar biasa

> **Pengakuan:** Apa yang dilakukan kode dengan baik

---

## Sumber Daya Pembelajaran

### Solidity/EVM
- [Dokumentasi Resmi](https://docs.soliditylang.org)
- [Best Practice Keamanan](https://consensys.github.io/smart-contract-best-practices/)
- [OpenZeppelin Contracts](https://docs.openzeppelin.com/contracts)
- [Ethereum Improvement Proposals (EIPs)](https://eips.ethereum.org)
- [Solidity by Example](https://solidity-by-example.org)
- [Smart Contract Weakness Classification](https://swcregistry.io)
- [Secureum Security Pitfalls](https://secureum.substack.com)
- [Trail of Bits - Building Secure Contracts](https://github.com/crytic/building-secure-contracts)

### Solana/Anchor
- [Dokumentasi Solana](https://docs.solana.com)
- [Anchor Framework](https://www.anchor-lang.com)
- [Anchor Book](https://book.anchor-lang.com)
- [Solana Cookbook](https://solanacookbook.com)
- [Solana Program Library (SPL)](https://spl.solana.com)
- [Neodyme Security Guide](https://github.com/neodyme-labs/solana-security-txt)
- [Sealevel Attacks](https://github.com/coral-xyz/sealevel-attacks)
- [Solana Security Best Practices](https://docs.solana.com/developing/programming-model/security)

### Keamanan Blockchain Umum
- [Rekt News (Database Eksploitasi)](https://rekt.news)
- [Analisis DeFi Hack](https://github.com/SunWeb3Sec/DeFiHackLabs)
- [Blockchain Security DB](https://github.com/openblocksec/blocksec-incidents)
- [Immunefi Bug Bounties](https://immunefi.com/learn/)

### Standar & Spesifikasi
- [Standar ERC](https://eips.ethereum.org/erc)
- [Solana Improvement Documents (SIMD)](https://github.com/solana-foundation/solana-improvement-documents)
- [Standar Token](https://ethereum.org/en/developers/docs/standards/tokens/)

---

*Kriteria review ini berkembang seiring dengan penemuan kerentanan baru, pembaruan framework, best practice komunitas, temuan audit, dan masukan developer.*

**Terakhir Diperbarui: 30 Januari 2026**