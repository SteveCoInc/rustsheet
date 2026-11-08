"""SKEW, SKEW.P and KURT's standardized-moment form over every eval spec case that calls them (T-4328, decision 0188 R4).

The engine since T-4328 builds the moment from stored x87 operations (each sum, product, quotient and root rounded to a
64-bit significand, then to a double) with the fourth power (z*z)*(z*z): a choice between structural forms, so it owes
the hold-out (R4.1) and a fresh sweep (R4.2: eval/accuracy/tables/{KURT,SKEW,SKEW.P}/r4-T-4328) but no `FITTED` tag.
Scored: the engine (this form), the once-rounded binary64 form the engine had before T-4328 (chained powers), and the
correctly rounded moment of the double inputs. The margin, 5 points (the default), and these cases (every spec case
whose text calls SKEW, SKEW.P or KURT, the t4974 adversarial probe left out as R4.2 evidence) were stated before
scoring. Results: T-4328's Outcome and statistics.md "Standardized moments use stored x87 operations (T-4328)".
"""

from __future__ import annotations

import math

import mpmath

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.kernels.devsq import _numbers
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg

_CASES = [
    "gen-m9/0072265d385d8ab2",
    "gen-m9/01523d247594952e",
    "gen-m9/05d44f6dd6cfc567",
    "gen-m9/0620505280d2ea6e",
    "gen-m9/06b4e8f5de409d5e",
    "gen-m9/097af1ca33638670",
    "gen-m9/09b47629f54b9817",
    "gen-m9/0a614cfe974b59c3",
    "gen-m9/0a9226016e8f5f33",
    "gen-m9/0a957119475760c5",
    "gen-m9/0bbab6206b5c9a07",
    "gen-m9/0c7fc447bc2eebfd",
    "gen-m9/1068c44afcd56d53",
    "gen-m9/11eb3ab4f4f8104c",
    "gen-m9/139684e68fb2fc3b",
    "gen-m9/146b46e74f2dac3b",
    "gen-m9/165db58d1540f014",
    "gen-m9/17c110a53924beb8",
    "gen-m9/1b31092a0d79e5f1",
    "gen-m9/1b60bf99d71424e7",
    "gen-m9/1cd3096e3e87c965",
    "gen-m9/1db18890916ee643",
    "gen-m9/1dce4774f5c0e736",
    "gen-m9/201d67f1d59436ed",
    "gen-m9/2138b909d8f810ea",
    "gen-m9/22656c225ff4f22a",
    "gen-m9/23d82b62e9a85bdd",
    "gen-m9/27601819397dd56e",
    "gen-m9/2801c1c11c21e134",
    "gen-m9/285d87cea1aaa73c",
    "gen-m9/288b99797f58f149",
    "gen-m9/290180acf017709c",
    "gen-m9/2a18b2f5a4dc43ac",
    "gen-m9/2f0d384b8ed0d392",
    "gen-m9/31f09e6c48ca245c",
    "gen-m9/320d51765af9b6b7",
    "gen-m9/329b177b5d5165ad",
    "gen-m9/334f3dc9ca937f10",
    "gen-m9/35e8168aae2b00f8",
    "gen-m9/38e4fc309fe199ea",
    "gen-m9/3a25a03eb94fcfbd",
    "gen-m9/3aa168259335be54",
    "gen-m9/3b7990bd4e3307ba",
    "gen-m9/3b8e9078261b5bd2",
    "gen-m9/3cf48801617a72ad",
    "gen-m9/3e98b26cf6713e77",
    "gen-m9/3fb9cecf2c23169f",
    "gen-m9/40425341c5034eed",
    "gen-m9/41248d1cb153c66e",
    "gen-m9/4250f880ab640c49",
    "gen-m9/43a13a091907b402",
    "gen-m9/43c6227e0ff368c7",
    "gen-m9/44a8841352dc48e9",
    "gen-m9/46ac3ef5e61583ca",
    "gen-m9/4ab2ba98d26c8a5e",
    "gen-m9/4bad303d32cc3f46",
    "gen-m9/4e180f54ff140abc",
    "gen-m9/4e55500bb8ad3cd8",
    "gen-m9/530c636749a85209",
    "gen-m9/5389dd4d5b5739d2",
    "gen-m9/5395b8a427764a3f",
    "gen-m9/5422bdbf173fc233",
    "gen-m9/55fc8c4ffbd7c7cc",
    "gen-m9/583ca2baea0bb30a",
    "gen-m9/589f42e67f4b4b8d",
    "gen-m9/58e3253bbd3a7a82",
    "gen-m9/5919660b7bb4d195",
    "gen-m9/59a16b5ab566ee5c",
    "gen-m9/5b6cc12a94f7bf14",
    "gen-m9/5be29fa316931f05",
    "gen-m9/5c6a23c4f6597f1c",
    "gen-m9/5cebac1f02bf30f2",
    "gen-m9/5ded7594e597e31e",
    "gen-m9/607e85cfee83a689",
    "gen-m9/60b0b6e69ad35a45",
    "gen-m9/623663cacea53bbc",
    "gen-m9/6376e27be7a05871",
    "gen-m9/65dc4e4da669f98a",
    "gen-m9/67a4c03afd63f988",
    "gen-m9/67f5e0fe790e6bec",
    "gen-m9/68d8ffb69ff739d8",
    "gen-m9/696af2c8cee2f57e",
    "gen-m9/6d16afd40600302b",
    "gen-m9/6e18550e9fc9c1b9",
    "gen-m9/73f053591a54bc43",
    "gen-m9/7674c89c824aca43",
    "gen-m9/775d053de498b8c7",
    "gen-m9/787ba8d801105a60",
    "gen-m9/7980ffb6095a340d",
    "gen-m9/79d87fc527927626",
    "gen-m9/7a819ffbbeff8fcc",
    "gen-m9/7bd219ec0c0c5e22",
    "gen-m9/7c20351562a2d71d",
    "gen-m9/7c7e5bff12d93365",
    "gen-m9/7f7ca726c3f68554",
    "gen-m9/7fd0a98183c6babf",
    "gen-m9/81bcdc0bec0c8e6b",
    "gen-m9/831f987e63e2412c",
    "gen-m9/85f48d5cdfb24555",
    "gen-m9/873f30d2bf76f8db",
    "gen-m9/899f9555e67da2d6",
    "gen-m9/89f0aee953a55a45",
    "gen-m9/8c018ae063ac131b",
    "gen-m9/8d3588d8df0812df",
    "gen-m9/8dd2312c96effb1b",
    "gen-m9/8f2ac25945af87da",
    "gen-m9/8fa8b572ebc1f001",
    "gen-m9/8fec7b207f4dd753",
    "gen-m9/91e167cd8d7eb652",
    "gen-m9/928d08e3e1eecb5f",
    "gen-m9/92d4fa29e5510f1e",
    "gen-m9/932b33b1ef8b7608",
    "gen-m9/941c409532603d32",
    "gen-m9/95efb37a9874f82e",
    "gen-m9/9735c5ab72fa5ecf",
    "gen-m9/984cba67bbe6972b",
    "gen-m9/9a1fa5366bc50918",
    "gen-m9/9bc247733a1ca9fa",
    "gen-m9/9cca17438a7e6f8a",
    "gen-m9/9d9ceba257f8001b",
    "gen-m9/9e2225ead8a6a07b",
    "gen-m9/9fa170a9f17624fd",
    "gen-m9/a013a45a8fbcab4f",
    "gen-m9/a2262ea466fcc142",
    "gen-m9/a4090567758520df",
    "gen-m9/a5342a7e24fd803e",
    "gen-m9/a54c738c697617c3",
    "gen-m9/a5828d3925575713",
    "gen-m9/a612e89321a6df9c",
    "gen-m9/a61e547a1cab994c",
    "gen-m9/a63d1c7d6f96c56d",
    "gen-m9/a790a4d4a6a82756",
    "gen-m9/a7bf0d8d81d33ba6",
    "gen-m9/abb7d565af89b849",
    "gen-m9/abc2939ebd1e2365",
    "gen-m9/acac71a65a5a1d1b",
    "gen-m9/afc00c8b70e78baf",
    "gen-m9/b4173fe25109a238",
    "gen-m9/b6593d7dd13f2dac",
    "gen-m9/b6a02893a8d6b151",
    "gen-m9/b72d952c5f52cb4d",
    "gen-m9/b73c6cc96e093f03",
    "gen-m9/b8ee55934c5b824c",
    "gen-m9/b9a32ce9a1fd0d71",
    "gen-m9/ba4ddb817a8e700e",
    "gen-m9/bbab6562e88b70e8",
    "gen-m9/bbdfc06cae0e0d71",
    "gen-m9/bc04c1f478c9896c",
    "gen-m9/bc40d37cb4d8301d",
    "gen-m9/bcb1105c61a2649d",
    "gen-m9/bcb24385baafb2f0",
    "gen-m9/c0681a8b150ce301",
    "gen-m9/c18dbf21f11093f3",
    "gen-m9/c2684066d8df263e",
    "gen-m9/c2974c061384fdd9",
    "gen-m9/c2d778463fe008ed",
    "gen-m9/c4635963fc926284",
    "gen-m9/c468f5046aadfbbc",
    "gen-m9/c6cfda6bd10cae93",
    "gen-m9/c7664e494e418395",
    "gen-m9/caa1eed48b1883fe",
    "gen-m9/cb61169f4c033d4b",
    "gen-m9/cb80ca09be0e12bc",
    "gen-m9/cde551ea787f713a",
    "gen-m9/ce312817dc9c7f0b",
    "gen-m9/d0854366830dcdee",
    "gen-m9/d193e8b4ab820f52",
    "gen-m9/d208bbafc3bc1997",
    "gen-m9/d37313406b9f6e55",
    "gen-m9/d5fd9e05e9e6f466",
    "gen-m9/d8266bc15533702a",
    "gen-m9/d8f44d556674c2fd",
    "gen-m9/d910f000b60e2eac",
    "gen-m9/d912671baa1364aa",
    "gen-m9/d9d55972b50e55e8",
    "gen-m9/e168be7f6d3f84c2",
    "gen-m9/e6cf3b20a014e6a3",
    "gen-m9/e882060cf4613302",
    "gen-m9/e8d55ec393460bb7",
    "gen-m9/ec2a1d9194a84396",
    "gen-m9/eccd03df42e40006",
    "gen-m9/eced4bb6a0234c9e",
    "gen-m9/ede0323c389fc703",
    "gen-m9/ee2497a5e683c5d9",
    "gen-m9/eedd629f3d69fdda",
    "gen-m9/f280df8ba95f96b9",
    "gen-m9/f2bd5f380ec002bf",
    "gen-m9/f34f66fb7f81e673",
    "gen-m9/f81bd3b6dc373e5c",
    "gen-m9/f82473275380882e",
    "gen-m9/f8bc18a0e45a0e1a",
    "gen-m9/fa25422d6f18c157",
    "gen-m9/fd9fa22d212080d6",
    "gen-m9/fec39efaba9284c3",
    "probes/aggregate-array-error-vs-ref-error-average-family",
    "probes/aggregate-direct-lambda-average-family",
    "probes/aggregate-error-precedence-stat-family",
    "probes/amordegrc-a76-bisect-a",
    "probes/amordegrc-a76-bisect-b",
    "probes/amordegrc-a76-bisect-c",
    "probes/amordegrc-a76-bisect-d",
    "probes/amordegrc-a76-bisect-f",
    "probes/amordegrc-a76-bisect-h",
    "probes/average-value-args",
    "probes/catalog/availability-22",
    "probes/catalog/availability-34",
    "probes/gen-m9-17c110a53924beb8-refill-1106",
    "probes/legacy-value-argument-intersection-b",
    "probes/multiarea-foreach",
    "probes/skew-p-bound",
    "probes/stat-argument-modes",
    "probes/stats-evidence-floor-t1623",
    "probes/t2436/165db58d1540f014",
    "probes/t2436/35e8168aae2b00f8",
    "probes/t2436/899f9555e67da2d6",
    "probes/t2436/8dd2312c96effb1b",
    "probes/t2436/f280df8ba95f96b9",
    "probes/t2475-stat-subexpressions-refill-1106",
    "probes/t2475-stat-subexpressions",
    "probes/t2496-fin-subexpressions",
    "probes/t2576-error-type-cell-paired",
    "probes/t2594-nested-aggregates-refill-1106",
    "probes/t2594-nested-aggregates",
    "probes/t4223-m9-statistical-array-kinds-a",
    "probes/t4223-m9-statistical-array-kinds-b",
]

_MIN = {"SKEW": 3, "SKEW.P": 3, "KURT": 4}


def _values(name: str, args: tuple[Arg, ...]) -> list[float]:
    values = _numbers(args)
    if len(values) < _MIN[name]:
        msg = f"{name} of fewer than {_MIN[name]} numbers is #DIV/0!"
        raise ValueError(msg)
    return values


def _plain(name: str, args: tuple[Arg, ...]) -> float:
    """The engine before T-4328: binary64 per operation, z*z*z and z*z*z*z chained left to right."""
    values = _values(name, args)
    n = float(len(values))
    mean = sum(values) / n
    m2 = 0.0
    for x in values:
        m2 += (x - mean) * (x - mean)
    if m2 == 0.0 or math.isinf(m2):
        msg = "zero variance is #DIV/0!"
        raise ValueError(msg)
    s = math.sqrt(m2 / (n if name == "SKEW.P" else n - 1.0))
    total = 0.0
    for x in values:
        z = (x - mean) / s
        total += z * z * z * z if name == "KURT" else z * z * z
    if name == "SKEW":
        return (n / ((n - 1.0) * (n - 2.0))) * total
    if name == "SKEW.P":
        return total / n
    return (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0)) * total - 3.0 * (n - 1.0) * (n - 1.0) / (
        (n - 2.0) * (n - 3.0)
    )


def _exact(name: str, args: tuple[Arg, ...]) -> float:
    """The moment of the double inputs at 60 digits, rounded once."""
    values = _values(name, args)
    with mpmath.workdps(60):
        xs = [mpmath.mpf(v) for v in values]
        n = mpmath.mpf(len(xs))
        mean = mpmath.fsum(xs) / n
        m2 = mpmath.fsum((x - mean) ** 2 for x in xs)
        if m2 == 0:
            msg = "zero variance is #DIV/0!"
            raise ValueError(msg)
        s = mpmath.sqrt(m2 / (n if name == "SKEW.P" else n - 1))
        power = 4 if name == "KURT" else 3
        total = mpmath.fsum(((x - mean) / s) ** power for x in xs)
        if name == "SKEW":
            out = n / ((n - 1) * (n - 2)) * total
        elif name == "SKEW.P":
            out = total / n
        else:
            out = n * (n + 1) / ((n - 1) * (n - 2) * (n - 3)) * total - 3 * (n - 1) ** 2 / ((n - 2) * (n - 3))
        return float(out)


def _fit(name: str) -> Fit:
    def plain(*args: Arg) -> float:
        return _plain(name, args)

    def exact(*args: Arg) -> float:
        return _exact(name, args)

    def candidates() -> list[Candidate]:
        return [
            candidate(REFERENCE, exact, "reference"),
            candidate("once-rounded binary64, chained powers (before T-4328)", plain, "plain"),
        ]

    return Fit(
        name=f"moments-{name.lower().replace('.', '-')}",
        description=f"{name}'s standardized-moment form over every spec case that calls it (T-4328)",
        cases=_CASES,
        pattern=rf"^={name.replace('.', chr(92) + '.')}\((.*)\)$",
        candidates=candidates,
    )


FITS = [_fit("SKEW"), _fit("SKEW.P"), _fit("KURT")]
