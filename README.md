## zkHack Challenge #4
*Challenge: https://zkhack.dev/events/puzzle4.html*

### Relevant Background

#### KZG Polynomial Commitment Scheme

The prover $\mathcal{P}$ wants to commit to a polynomial of the form:
$$
f(x) = f_0 + f_1 x + \dots + f_d x^d
$$

---

**Setup**

KZG follows a trusted setup. This means that a trusted party samples $\tau \in_{R} \mathbb{F}_p$ generates a structured reference string (SRS):
$$
gp := \{ H_0, H_1, \dots, H_d \} = \{ g^{\tau^0}, g^{\tau^1}, \dots, g^{\tau^d} \}
$$

> **Note:** $\tau$ must be destroyed after the setup phase in order for the protocol to remain secure.

---

**Commit**

$\mathcal{P}$ computes the following:

$$
\begin{align*}
\text{comm}(f) = g^{f(\tau)} &= g^{f_0 + f_1 \tau + f_2 \tau^2 + \dots + f_d \tau^d} \\
                            &= \left(g^{\tau^0}\right)^{f_0} \cdot \left(g^{\tau^1}\right)^{f_1} \cdot \left(g^{\tau^2}\right)^{f_2} \cdots \left(g^{\tau^d}\right)^{f_d} \\
                            &= H_0^{f_0} \cdot H_1^{f_1} \cdot \dots \cdot H_d^{f_d}
\end{align*}
$$

> **Note:** $\text{comm}(f)$ is a single group element $g^{f(\tau)} \in \mathbb{G}$, not a vector. The prover can compute this efficiently in linear time using multiexponentiation.

---

*Lagrange Interpolation:* 

If the polynomial is defined by a set of evaluations: $\{ (x_0, y_0), \dots, (x_n, y_n)\}$, then it takes exponential time to compute the commitment by converting to coefficient form. In this case, it is simpler to compute the commitment by using interpolation.

We define the Lagrange basis polynomials $L_0(x), L_1(x), \dots, L_n(x)$ as:

$$
L_i(x) = \prod_{\substack{0 \leq j \leq n \\ j \ne i}} \frac{x - x_j}{x_i - x_j}
$$

These satisfy:

$$
L_i(x_j) = \begin{cases}
1 & \text{if } i = j \\
0 & \text{if } i \ne j
\end{cases}
$$

$L_i(x_j)$ can be thought of as an indicator.  
So we can express the polynomial $f(x)$ as:

$$
\tilde{f}(x) = \sum_{i=0}^{n} f(x_i) \cdot L_i(x)
$$

In particular, we can evaluate this at the secret value $\tau$ from the trusted setup:

$$
f(\tau) = \sum_{i=0}^{n} f(x_i) \cdot L_i(\tau)
$$

Then, the KZG commitment becomes:

$$
\text{comm}(f) = g^{f(\tau)} = g^{\sum_{i=0}^{n} f(x_i) \cdot L_i(\tau)} = \prod_{i=0}^{n} \left( g^{L_i(\tau)} \right)^{f(x_i)}
$$

> **Note:** The evaluation domain $\mathcal{D} = \{ x_0, x_1, \dots, x_{n-1} \}$ is the set of points where the polynomial $f$ is known (evaluated). In KZG commitments, $\mathcal{D}$ is chosen to be a multiplicative subgroup of $\mathbb{F}_p^*$ (e.g., the set of $n$-th roots of unity). This structure enables the use of the Fast Fourier Transform (FFT) and its inverse (IFFT) to perform fast conversions between coefficient and evaluation representations, as well as efficient polynomial operations. As a result, instead of the naive $O(n^2)$ complexity for evaluation and interpolation, these operations can be done in $O(n \log n)$ time. Further, evaluating the polynomial at a single arbitrary point $z$ using barycentric interpolation or similar methods can be done in $O(n)$ time.

---

**Open**

We know that the following always holds:
$$
f(x) - f(\gamma) = (x - \gamma) \cdot q(x) \quad \text{(since }\gamma \text{ is a root of } f(x) - f(\gamma))
$$
We will use this to verify the commitment.

To prove that the committed polynomial $f(x)$ evaluates to a claimed value $y$ at a challenge point $\gamma \in \mathbb{F}_p$, the prover $\mathcal{P}$ computes an opening proof $(y, \pi)$:

Compute the polynomial with degree $d-1$:
$$
q(x) = \frac{f(x) - f(\gamma)}{(x - \gamma)}
$$

Compute the commitment to $h(x)$:
$$
\pi = \text{comm}(h) = g^{h(\tau)} = \prod_{i=0}^d H_i^{h_i}
$$

> **Note:** The polynomial division is well-defined since $ (x - \gamma) $ divides $ f(x) - f(\gamma) $ exactly.

---

**Verify**

The verifier $\mathcal{V}$ performs the following check:

$$
e(\text{comm}(f) - g^{y}, g) \stackrel{?}{=} e(\pi, g^{\tau - \gamma})
$$

The prover claims:

- $\text{comm}(f) = g^{f(\tau)}$
- $\pi = g^{q(\tau)}$, where $q(x) = \frac{f(x) - f(\gamma)}{x - \gamma}$

Recall from basic polynomial division:

$$
f(x) - f(\gamma) = (x - \gamma) \cdot q(x)
$$

Evaluating at $x = \tau$ gives:

$$
f(\tau) - f(\gamma) = (\tau - \gamma) \cdot q(\tau)
$$

Exponentiating both sides:

$$
g^{f(\tau) - f(\gamma)} = g^{(\tau - \gamma) \cdot q(\tau)} = (g^{q(\tau)})^{\tau - \gamma}
$$

Now using the bilinearity of the pairing $e : \mathbb{G}_1 \times \mathbb{G}_1 \to \mathbb{G}_T$:

$$
e(g^{f(\tau) - f(\gamma)}, g) = e(g^{q(\tau)}, g^{\tau - \gamma}) = e(\pi, g^{\tau - \gamma})
$$

So the verifier computes both sides of:

$$
e(\text{comm}(f) - g^y, g) \stackrel{?}{=} e(\pi, g^{\tau - \gamma})
$$

> **Security Assumption:**
> This protocol is sound under the *q-Strong Bilinear Diffie-Hellman (q-SBDH) Assumption*.

> While the *computational* soundness of KZG commitments relies on the **q-SBDH assumption**, the **intuition** can be understood through the **Schwartz-Zippel lemma**.
>
> Suppose a dishonest prover claims that a committed polynomial $f$ evaluates to $y$ at some point $\gamma \in \mathbb{F}_p$, even though $f(\gamma) \ne y$. To convince the verifier, the prover must construct a quotient polynomial $q(x)$ such that:
> $$
> f(x) - y = (x - \gamma) \cdot q(x)
> $$
> But this identity implies that $f(\gamma) = y$, which contradicts the assumption. So the prover has constructed a **false polynomial identity**.
>
> Now consider evaluating both sides at the secret point $\tau$:
> $$
> f(\tau) - y \stackrel{?}{=} (\tau - \gamma) \cdot q(\tau)
> $$
> For the pairing check to pass, this must hold. But since the underlying identity is false, the left and right sides define **distinct polynomials** of degree at most $d$.
>
> The **Schwartz-Zippel lemma** says that two distinct degree-$d$ univariate polynomials over $\mathbb{F}_p$ agree at a randomly chosen point with probability at most $d/p$. So unless the prover can guess the secret $\tau$, this equality holds with **negligible probability**.
>
> > **Lemma (Schwartz-Zippel, univariate):**  
> > Let $P(x)$ be a nonzero polynomial of degree at most $d$ over $\mathbb{F}_p$, and let $\gamma \in \mathbb{F}_p$ be chosen uniformly at random. Then:
> > $$
> > \Pr[P(\gamma) = 0] \le \frac{d}{p}
> > $$
>
> In practice, KZG avoids relying on this probabilistic argument directly by cryptographically hiding $\tau$ via the q-SBDH assumption. But the Schwartz-Zippel lemma still provides useful intuition: **if a fake opening passes the pairing check, it implies a false polynomial identity that "magically" holds at $\tau$ — which is extremely unlikely unless the prover can break the underlying hardness assumption.**

---

**Batching**

KZG polynomial commitments support **batch opening proofs**, allowing a prover to efficiently prove multiple evaluation points with a *single* proof, reducing verification cost.

Suppose the prover wants to prove that:

$$
f(\gamma_1) = y_1, \quad f(\gamma_2) = y_2, \quad \dots, \quad f(\gamma_k) = y_k
$$

instead of providing $k$ separate opening proofs, the prover computes a **single aggregated proof** $\pi$ as follows:

1. Choose random (or Fiat-Shamir derived) challenge scalars $c_1, \dots, c_k \in \mathbb{F}_p$.
2. Define the polynomial:

$$
R(x) := \sum_{i=1}^k c_i \cdot \frac{f(x) - y_i}{x - \gamma_i}
$$

which has degree at most $d-1$ since each term is a quotient polynomial.

3. Compute the batch opening proof:

$$
\pi := g^{R(\tau)} = \text{comm}(R)
$$

The verifier then checks the **aggregated pairing equation**:

$$
e\left( \text{comm}(f) \cdot \prod_{i=1}^k g^{-c_i y_i}, g \right) \stackrel{?}{=} e\left( \pi, \prod_{i=1}^k g^{c_i (\tau - \gamma_i)} \right)
$$

Using bilinearity, this equality verifies *all* claimed evaluations simultaneously.

---

 Each evaluation check corresponds to verifying

  $$
  e(g^{f(\tau) - y_i}, g) \stackrel{?}{=} e(g^{q_i(\tau)}, g^{\tau - \gamma_i})
  $$

The batching combines them into a single linear combination weighted by $c_i$.

**Why random scalars \(c_i\) are needed in batching:**  
The challenge scalars \(c_i\) ensure soundness by preventing a dishonest prover from exploiting linearity to cancel out errors across multiple proofs. Even though the evaluation points \(\gamma_i\) are distinct, without random weighting, a fake combined proof might pass verification by cleverly cancelling inconsistencies. The random \(c_i\) create a randomized linear combination of the quotient polynomials, forcing the prover to be correct on *all* evaluations or fail the batch check with high probability.

---

#### Blinding Schemes

Let the evaluation domain be:
$$
\mathcal{D} = \{x_0, x_1, \dots, x_{n-1}\}
$$
and define the vanishing polynomial:
$$
Z_{\mathcal{D}}(x) = \prod_{i=0}^{n-1} (x - x_i)
$$
which satisfies $Z_{\mathcal{D}}(x_i) = 0$ for all $x_i \in \mathcal{D}$.

Given a polynomial $f(x)$ of degree less than $n$, the prover samples a random scalar $r \in \mathbb{F}_p$ and forms the blinded polynomial:
$$
f'(x) = f(x) + r \cdot Z_{\mathcal{D}}(x)
$$

Because $Z_{\mathcal{D}}(x_i) = 0$ on the evaluation domain, the blinded polynomial satisfies:
$$
f'(x_i) = f(x_i) \quad \text{for all } x_i \in \mathcal{D}
$$

Thus, the polynomial evaluations on the domain remain unchanged, preserving correctness of the committed values. However, $f'(x)$ hides the original polynomial $f(x)$ outside $\mathcal{D}$, and the commitment to $f'(x)$ reveals nothing about $f(x)$ beyond its evaluations on $\mathcal{D}$.

---

#### The Hidden In Plain Sight Protocol

This protocol enables anonymous transactions within a shielded pool of 1000 accounts, where sensitive data like recipient addresses is hidden. 
Each transaction commits to a 256-bit recipient address using a **blinded KZG-style polynomial commitment**. 

**Blinding Scheme**

The 256-bit recipient address is encoded as a vector of 32 bytes. Each byte is interpreted as a scalar in the BLS12-381 field and becomes a coefficient in a degree-31 polynomial:
$$
P(x) = a_0 + a_1x + \dots + a_{31}x^{31}
$$

An evaluation domain $H = \{\omega^0, \omega^1, \dots, \omega^{n-1}\}$ is chosen, where $\omega$ is a primitive $n$-th root of unity such that $\omega^n = 1$. The associated vanishing polynomial is:
$$
Z_H(x) = x^n - 1
$$
which vanishes on all elements of $H$, i.e., $Z_H(h) = 0$ for all $h \in H$.

To hide the address, the sender samples two secret blinding scalars $b_0, b_1 \in \mathbb{F}_p$ and computes the blinded polynomial:
$$
Q(x) = P(x) + (b_0 + b_1x) \cdot Z_H(x)
$$

This blinding preserves the evaluations of $P(x)$ on $H$, but hides its coefficients outside of the domain, since $Z_H(x)$ vanishes on $H$ but not elsewhere.

The sender then computes KZG-style commitment to $Q(x)$.
We are provided with two openings: $\{(z_1, Q(z_1)), (z_2, Q(z_2))\}$

### The Exploit

For each commitment, we are giving two openings: $\{(z_1, Q(z_1)), (z_2, Q(z_2))\}$.

This gives:
$$
Q(z_1) = P(z_1) + (b_0 + b_1z_1) \cdot z_H(z_1)
$$
$$
Q(z_2) = P(z_2) + (b_0 + b_1z_2) \cdot z_H(z_2)
$$

Observe that we can compute $P(x)$ for every account.
We also know $z_H(x)$.

This gives the following system of equations:

$$\left\{
\begin{array}{l}
\frac{Q(z_1) - P(z_1)}{Z_H(z_1)} = b_0 + b_1 z_1 \\
\frac{Q(z_2) - P(z_2)}{Z_H(z_2)} = b_0 + b_1 z_2 \\
\end{array}
\right.$$

Here, we have two equations and two unknowns so we can solve the linear system for $b_0$ and $b_1$.

From here, we seek to recontruct $Q(x)$ so we can compute the commitment comm(Q).

By expanding, we observe the following:

$$
Q(x) = P(x) + (b_0 + b_1x) \cdot Z_H(x)
$$
$$
\begin{align*}
\implies q_0 + q_1x_1 + \dots + q_nx_n^n + q_{n+1}x_{n+1}^{n+1} &= (p_0 + p_1 x_1\dots + p_{n-1}x^{n-1}) + b_0 x^n + b_1 x^{n+1} - b_0 - b_1 x \\
&= (p_0 - b_0) + (p_1 - b_1) x + \dots + p_{n-1}x^{n-1} + b_0 x^n + b_1  x^{n+1}
\end{align*}
$$

We know that, if two polynomials are the same, then their coefficients are the same:  
$q_0 = p_0 - b_0$  
$q_1 = p_1 - b_1$  
$q_i = p_i \quad \forall i \in [n] \setminus \{1\}$
$q_n = b_0$  
$q_{n+1} = b_1$  

All of these values are known. Thus, we can compute $Q(x)$ and comm(Q).

Since we have to compute $P(x)$ for every account, we repeat this for every account in the pool until we compute the matching commitment.

---

#### Commands

```rust
cargo run --bin verify-hidden-in-plain-sight
```
