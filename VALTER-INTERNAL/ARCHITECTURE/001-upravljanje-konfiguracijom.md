### **ADR-001: Hijerarhijsko Upravljanje Konfiguracijom (Verzija 2)**

**Status:** Potvrđeno
**Datum:** 2026-01-07
**Arhitekt:** Gemini

---

## 1. Kontekst

VALTER zahtijeva robustan sustav konfiguracije koji razdvaja razvoj od produkcije, osigurava da su produkcijske binarne datoteke samostalne i predvidljive, ali istovremeno pruža naprednim korisnicima mogućnost da nadjačaju ugrađene postavke koristeći sistemske varijable okruženja. Ključno je spriječiti djelomično nadjačavanje koje bi moglo dovesti do nekonzistentnog stanja sustava (npr. korištenje novog modela sa starim API ključem).

## 2. Odluka

Implementirat ćemo **dvofazni sustav konfiguracije** koji se temelji na ugradnji vrijednosti u vrijeme kompajliranja, s opcionalnim, strogo kontroliranim mehanizmom za nadjačavanje u vrijeme izvršavanja.

### Faza 1: Vrijeme Kompilacije (Compile-Time)

1.  **Primarni Izvor:** Datoteka `.env` u korijenu projekta je jedini izvor konfiguracije za vrijeme kompajliranja.
2.  **Ugradnja:** `build.rs` skripta će pročitati `.env` datoteku i ugraditi (ispeći) vrijednosti varijabli direktno u binarnu datoteku.
3.  **Stroga Validacija:** Build će biti **prekinut s greškom** ako obavezne varijable (definirane niže) nedostaju u `.env` datoteci. Ne postoje "hardkodirane" fallback vrijednosti unutar Rust koda za ključne podatke poput API ključeva.
4.  **Prekidač za Nadjačavanje:** Vrijednost varijable `ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME` iz `.env` datoteke bit će ugrađena kao `bool` zastavica unutar binarne datoteke. Ona odlučuje hoće li aplikacija uopće pokušati čitati sistemske varijable pri pokretanju.

### Faza 2: Vrijeme Izvršavanja (Runtime)

Ponašanje aplikacije pri pokretanju ovisi o ugrađenoj zastavici za nadjačavanje.

#### Scenarij A: Nadjačavanje Onemogućeno (`..._ENABLE_OVERRIDES...` = `false` ili nije definirano)
*   Aplikacija **isključivo i uvijek** koristi vrijednosti koje su ugrađene tijekom kompajliranja.
*   Sve sistemske varijable s prefiksom `VALTER_` se u potpunosti ignoriraju. Ovo je zadano i najsigurnije ponašanje.

#### Scenarij B: Nadjačavanje Omogućeno (`..._ENABLE_OVERRIDES...` = `true`)
*   Aplikacija je svjesna da *može* biti nadjačana. Slijedi strogi, hijerarhijski proces provjere:
    1.  **Provjera "Escape" Varijable:** Prvo se provjerava postojanje sistemske varijable `VALTER_IGNORE_ENV_DURING_RUNTIME`.
        *   Ako je postavljena na `true`, cijeli proces nadjačavanja se prekida. Aplikacija nastavlja koristiti **ugrađene (compile-time) vrijednosti**. Korisničko sučelje će prikazati informativnu poruku o tome.
    2.  **"Sve ili Ništa" Provjera:** Ako "escape" varijabla nije postavljena, sustav **mora** pronaći **svih šest** definiranih `VALTER_` sistemskih varijabli:
        *   `VALTER_PROVIDER`
        *   `VALTER_GEMINI_API_KEY`
        *   `VALTER_MODEL`
        *   `VALTER_RPM`
        *   `VALTER_SEARCH_API_KEY`
        *   `VALTER_SEARCH_CX`
    3.  **Evaluacija Stanja:**
        *   **Ako su SVIH ŠEST pronađene:** Aplikacija odbacuje ugrađene vrijednosti i koristi isključivo vrijednosti iz sistemskog okruženja.
        *   **Ako ijedna od šest nedostaje:** Aplikacija **ne koristi nijednu** sistemsku varijablu i nastavlja koristiti **ugrađene** vrijednosti. Međutim, u ovom slučaju, sustav ulazi u stanje greške. Korisničko sučelje će prikazati **blokirajuće upozorenje** koje navodi koje točno varijable nedostaju u sistemskom okruženju.
    4.  **Ponovna Provjera:** Unutar upozorenja na sučelju, postojat će gumb "Ponovno provjeri" koji će pokrenuti ponovnu evaluaciju sistemskih varijabli (koraci B1 i B2), omogućujući korisniku da ispravi svoje okruženje bez ponovnog pokretanja aplikacije.

## 3. Obrazloženje

Ovaj dizajn postiže sve ciljeve:
*   **Predvidljivost:** Zadani build je hermetičan i neovisan o okruženju u kojem se pokreće.
*   **Stroga Sigurnost:** Pravilo "sve ili ništa" sprječava nekonzistentno stanje gdje bi se, na primjer, koristio novi `VALTER_MODEL` s API ključem ugrađenim iz starog `.env` fajla.
*   **Kontrola i Jasnoća:** Korisnik je uvijek jasno obaviješten o stanju konfiguracije. Ne postoji "tiha" greška. Upozorenje je eksplicitno i nudi rješenje (ili postavljanje varijabli ili korištenje "ignore" zastavice).
*   **DX i Fleksibilnost:** Razvojni tim zadržava jednostavnost `.env` datoteke, dok napredni korisnici dobivaju moćan mehanizam za nadjačavanje.

## 4. Detaljna Specifikacija Varijabli

| Varijabla u `.env` | Ugrađeno kao | Obavezna? | Fallback (u `build.rs`) | Opis |
|---|---|---|---|---|
| `VALTER_PROVIDER` | `&'static str` | Da | `"gemini"` | Pružatelj AI usluge. |
| `VALTER_GEMINI_API_KEY` | `&'static str` | Da | **Nema** (Build puca) | API ključ za Gemini. |
| `VALTER_MODEL` | `&'static str` | Da | `"gemini-1.5-flash-lite"` | Specifični AI model. |
| `VALTER_RPM` | `&'static str` | Da | `"30"` | Maksimalan broj zahtjeva u minuti. |
| `VALTER_SEARCH_API_KEY`| `&'static str` | Ne | `""` (prazan string) | API ključ za Google Search. |
| `VALTER_SEARCH_CX` | `&'static str` | Ne | `""` (prazan string) | ID za Google Custom Search. |
| `ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME` | `bool` | Ne | `false` | Glavni prekidač za omogućavanje runtime nadjačavanja. |
