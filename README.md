# testauskameli
![](./img/kameli.png)

Testausserverin siistein botti

## Ominaisuuksia

### Kieliajurit
Kun pingaat kamelin viestiin, jossa on tuetun kielen koodipalikka
kameli yrittää ajaa kyseisen koodin ja tulostaa vastausviestiin sen tulosteen ja mahdolliset virheet.

Tuetut kielet:
- C
- Haskell
- Idris2
- Common Lisp

### Meemi generaattori
Kun pingaat kamelin viestiin, joka toteutaa regexin `no\s+(.*)?\?`,
kameli lähettää sinulle hienon meemin

### Echo
Kun pingaat kamelin viestiin, joka alkaa sanalla "echo", kameli lähettää sinulle loput viestistä.
Hyödyllinen testaukseen.

### WHOIS
Kun pingaat kamelin viestiin, joka on muotoa "whois [domain]", kameli lähettää sinulle annetun domainin
whois-tiedot.

### h264ify
Kameli pyrkii automaattisesti tunnistamaan videot, joiden toistaminen ei onnistu discordissa ja muuttamaan ne muotoon,
jossa discord kykeene toistamaan niitä

## Vaatimuksia
### Kamelin vaatimat ohjelmistot ja muut

#### .env
`.env`-nimisessä tiedostossa tulee olla seuraavat muuttujat määriteltyinä:
```
DISCORD_TOKEN=
GHC_ARGS=
KAMELI_TIMELIMIT=
KAMELI_RUNUSER=nobody # voidaan tarvittaessa muuttaa
FONT_PATH=
```

#### /etc/security/limits.conf
Kameli hyödyntää (Linux-)kernelin valvomien resurssirajoituksien asettamista
Asettaminen tapahtuu muokkaamalla `/etc/security/limits.conf` tiedostoa, root-käyttäjän oikeuksilla.

Jos kameli on konfiguroitu käyttämään `nobody`-käyttäjää ohjelmistojen ajamiseen, tulee `/etc/security/limits.conf` tiedoston näyttää jotakuinkin tältä:
```
nobody  hard    nproc   50 # Vanha KAMELI_PROCESSLIMIT
nobody  hard    locks   10 # Vanha KAMELI_FILELIMIT
nobody  hard    as  1024000 # Vanha KAMELI_MEMLIMIT
```

#### Ohjelmistot
Seuraavat ohjelmistot tulevat olla ladattuna ja kamelin käytettävissä:
1. timeout
2. sudo:
    `/etc/sudoers`:sta tulisi löytyä vastaava configuraatio:
    ```
    %sudo   ALL=(nobody) NOPASSWD: ALL
    ```
    Jossa `nobody` vastaa `$KAMELI_RUNUSER`-muuttujaa
3. snippettien vaatimat ohjelmistot:
    * ghc
    * gcc
    * idris2
    * chez
    * clisp
    * ffmpeg

Kohdan 3 ohjelmistojen puuttuessa, kameli ilmoittaa puutteista ja keskeyttää käynnistymisen

#### Muut
Kamelia ajavalle koneelle tulee olla luotuna `$KAMELI_RUNUSER` mukainen käyttäjä.
On suositeltavaa, ettei tällä käyttäjällä ole mitään oikeuksia.

Helpoiten tämä onnistuu käyttämällä unixeista jo vakiona löytyvää `nobody`-käyttäjää

## Tarkoitus
Kamelin tarkoitus on olla sekä viihdyttävä, että hyödyllinen.
Jatkossa kameliin voi lisätä esim. meemi-generaatio-komentoja.

Lyh. Kameliin on tarkoitus lisätä kaikki ne komennot, jotka halutaan,
mutta jotka eivät sovi [testauskoiraan](https://github.com/Testausserveri/testauskoira-rs)
