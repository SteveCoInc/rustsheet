# T-4552 engine predictions (written before the excel-graph fill)

probes/t4552-bessel-tiny-x-cut, as the engine at origin/main computed it:
- A1:A5 (the build of x = 2^-1021): 1, 1, FALSE, TRUE, TRUE.
- B1:B4 BESSELK(2^-1021, 0 and 1), both builds: 707.8192028722641, 2.247116418577895E+307 (the cut is `x < 2^-1021`).
- C, D, E (BESSELJ, BESSELI, BESSELY) at every x of rows 1:8 answer, none `#NUM!`: J0 1.00000000283141, I0 1,
  Y0 -450.6117008295493 (-451.0529720308492 at 2^-1022), J1 and I1 about x/2, Y1 about -2/(pi x).
