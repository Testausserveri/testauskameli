# testauskameli
![](./img/kameli.png)

Testausserverin siistein botti

## Ominaisuuksia

### Haskell ajuri
Kun pingaat kamelin viestiin, jossa on Haskell-koodipalikka
kameli yrittää ajaa kyseisen koodin ja tulostaa vastausviestiin sen tulosteen ja mahdolliset virheet.

## Vaatimuksia
### Kamelin vaatimat ohjelmistot ja muut

#### .env
`.env`-nimisessä tiedostossa tulee olla seuraavat muuttujat määriteltyinä:
```
DISCORD_TOKEN=
KAMELI_MEMLIMIT=
KAMELI_TIMELIMIT=
KAMELI_RUNUSER=
```

#### Ohjelmistot
Seuraavat ohjelmistot tulevat olla ladattuna ja kamelin käytettävissä:
1. s6 (tark. `s6-softlimit`)
2. sudo:
    `/etc/sudoers`:sta tulisi löytyä vastaava configuraatio:
    ```
    %sudo   ALL=(runhaskell) NOPASSWD: ALL
    ```
    Jossa `runhaskell` vastaa `$KAMELI_RUNUSER`-muuttujaa
3. runghc (yleisesti samassa paketissa `ghc`:n kanssa)

#### Muut
Kamelia ajavalle koneelle tulee olla luotuna `$KAMELI_RUNUSER` mukainen käyttäjä.
On suositeltavaa, ettei tällä käyttäjällä ole mitään oikeuksia.

## Tarkoitus
Kamelin tarkoitus on olla sekä viihdyttävä, että hyödyllinen.
Jatkossa kameliin voi lisätä esim. meemi-generaatio-komentoja.

Lyh. Kameliin on tarkoitus lisätä kaikki ne komennot, jotka halutaan,
mutta jotka eivät sovi [testauskoiraan](https://github.com/Testausserveri/testauskoira-rs)
