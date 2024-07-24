# Blind Diffie-Hellman Key Exchange (blind ecash)

> [!IMPORTANT]
> This document was taken from [here](https://gist.github.com/RubenSomsen/be7a4760dd4596d06963d67baf140406).

The goal of this protocol is for Bob to get Alice to perform a Diffie-Hellman key exchange blindly, such that when the unblinded value is returned, Alice recognizes it as her own, but can’t distinguish it from others (i.e. similar to a blind signature).

```
Alice:
A = a*G
return A

Bob:
Y = hash_to_curve(secret_message)
r = random blinding factor
B'= Y + r*G
return B'

Alice:
C' = a*B'
  (= a*Y + a*r*G)
return C'

Bob:
C = C' - r*A
 (= C' - a*r*G)
 (= a*Y)
return C, secret_message

Alice:
Y = hash_to_curve(secret_message)
C == a*Y

If true, C must have originated from Alice
```

I unearthed this protocol from a seemingly long forgotten [cypherpunk mailing list post](http://cypherpunks.venona.com/date/1996/03/msg01848.html) by David Wagner from 1996 (edit: perhaps not as forgotten as I thought, as [Lucre](https://github.com/benlaurie/lucre) is an implementation of it). It was devised as an alternative to RSA blinding in order to get around the now-expired patent by David Chaum. As in all ecash protocols, the `secret_message` is remembered by `Alice` in order to prevent double spends.

One benefit of this scheme is that it's relatively straightforward to perform in a threshold setting (only requires curve multiplication). One downside is that validation is more involved than simply checking a signature, as this step requries repeating the Diffie-Hellman Key Exchange.

The protocol also has one additional weakness that can be addressed. Bob can't be certain that `C'` was correctly generated and thus corresponds to `a*B'` . Alice can resolve this by also supplying a discrete log equality proof (DLEQ), showing that `a` in `A = a*G` is equal to `a` in `C' = a*B'`. This equality can be proven with a relatively simple Schnorr signature, as described below.

```
(These steps occur once Alice returns C')

Alice:
 r = random nonce
R1 = r*G
R2 = r*B'
 e = hash(R1,R2,A,C')
 s = r + e*a
return e, s

Bob:
R1 = s*G - e*A 
R2 = s*B'- e*C'
e == hash(R1,R2,A,C')

If true, a in A = a*G must be equal to a in C' = a*B'
```

Thanks to Eric Sirion, Andrew Poelstra, and Adam Gibson for their helpful comments.
