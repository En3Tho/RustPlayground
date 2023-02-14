using System.Text.RegularExpressions;
using BenchmarkDotNet.Attributes;
using RustExport;

namespace RustImport;

public unsafe class RSRegexBenchmark
{
    [Params(
        new byte[] { 84, 104, 114, 101, 97, 100, 67, 111, 117, 110, 116, 58, 32, 32, 32, 32, 32, 32, 52, 50, 0 },
        new byte[] { 32, 68, 66, 71, 32, 32, 32, 73, 68, 32, 32, 32, 32, 32, 79, 83, 73, 68, 32, 84, 104, 114, 101, 97, 100, 79, 66, 74, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 83, 116, 97, 116, 101, 32, 71, 67, 32, 77, 111, 100, 101, 32, 32, 32, 32, 32, 71, 67, 32, 65, 108, 108, 111, 99, 32, 67, 111, 110, 116, 101, 120, 116, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 68, 111, 109, 97, 105, 110, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 67, 111, 117, 110, 116, 32, 65, 112, 116, 32, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0 },
        new byte[] { 32, 32, 49, 49, 32, 32, 32, 32, 53, 32, 32, 32, 32, 32, 32, 32, 49, 51, 32, 48, 48, 48, 48, 55, 70, 67, 51, 48, 67, 48, 49, 56, 54, 55, 48, 32, 32, 49, 48, 50, 48, 50, 50, 48, 32, 80, 114, 101, 101, 109, 112, 116, 105, 118, 101, 32, 32, 48, 48, 48, 48, 55, 70, 67, 51, 51, 68, 68, 51, 65, 69, 49, 56, 58, 48, 48, 48, 48, 55, 70, 67, 51, 51, 68, 68, 51, 67, 57, 65, 48, 32, 48, 48, 48, 48, 55, 70, 67, 68, 57, 70, 69, 52, 52, 48, 54, 48, 32, 45, 48, 48, 48, 48, 49, 32, 85, 107, 110, 32, 40, 84, 104, 114, 101, 97, 100, 112, 111, 111, 108, 32, 87, 111, 114, 107, 101, 114, 41, 32, 0 })]
    public byte[] Utf8 { get; set; } = null!;

    [Benchmark]
    public bool Rust()
    {
        fixed (byte* ch = Utf8)
        {
            return Interop.call_regex(ch);
        }
    }
}

public partial class CSRegexBenchmark
{
    [GeneratedRegex(@"\s*((?:XXXX)|-?[0-9]+)\s+(-?[0-9]+)\s+((?:0[xX])?[0-9a-fA-F]+)\s+([0-9a-fA-F]{16})\s+((?:0[xX])?[0-9a-fA-F]+)\s+([^\s].*[^\s])\s+([0-9a-fA-F]{16}:[0-9a-fA-F]{16})\s+([0-9a-fA-F]{16})\s+(-?[0-9]+)\s+([^\s].*[^\s])\s*([^\s].*[^\s])*\s*")]
    public static partial Regex CSharpRegex();

    [Params(
        "ThreadCount:      42",
        " DBG   ID     OSID ThreadOBJ           State GC Mode     GC Alloc Context                  Domain           Count Apt Exception",
        "  11    5       13 00007FC30C018670  1020220 Preemptive  00007FC33DD3AE18:00007FC33DD3C9A0 00007FCD9FE44060 -00001 Ukn (Threadpool Worker) ")]
    public string String { get; set; } = null!;

    [Benchmark]
    public bool CSharp() => CSharpRegex().IsMatch(String);
}

