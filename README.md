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
DISCORD_TOKEN=kamelindiscordtoken
```

#### Ohjelmistot
Seuraavat ohjelmistot tulevat olla ladattuna ja kamelin käytettävissä:
1. s6 (tark. `s6-softlimit`)
2. sudo
3. runghc

#### Muut
Kamelia ajavalle koneelle tulee olla luotuna `runhaskell` niminen käyttäjä.
On suositeltavaa, ettei tällä käyttäjällä ole mitään oikeuksia.

## Tarkoitus
Kamelin tarkoitus on olla sekä viihdyttävä, että hyödyllinen.
Jatkossa kameliin voi lisätä esim. meemi-generaatio-komentoja.

Lyh. Kameliin on tarkoitus lisätä kaikki ne komennot, jotka halutaan,
mutta jotka eivät sovi [testauskoiraan](https://github.com/Testausserveri/testauskoira-rs)
