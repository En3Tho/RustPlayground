// See https://aka.ms/new-console-template for more information

using System.Reflection;
using System.Runtime.CompilerServices;
using BenchmarkDotNet.Running;
using RustExport;
using RustImport;

unsafe void CallStr()
{

    var x1 = new byte[] { 84, 104, 114, 101, 97, 100, 67, 111, 117, 110, 116, 58, 32, 32, 32, 32, 32, 32, 52, 50, 0 };

    var x2 = new byte[]
    {
        32, 68, 66, 71, 32, 32, 32, 73, 68, 32, 32, 32, 32, 32, 79, 83, 73, 68, 32, 84, 104, 114, 101, 97, 100, 79, 66,
        74, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 83, 116, 97, 116, 101, 32, 71, 67, 32, 77, 111, 100, 101, 32,
        32, 32, 32, 32, 71, 67, 32, 65, 108, 108, 111, 99, 32, 67, 111, 110, 116, 101, 120, 116, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 68, 111, 109, 97, 105, 110, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 67, 111, 117, 110, 116, 32, 65, 112, 116, 32, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0
    };

    var x3 = new byte[]
    {
        32, 32, 49, 49, 32, 32, 32, 32, 53, 32, 32, 32, 32, 32, 32, 32, 49, 51, 32, 48, 48, 48, 48, 55, 70, 67, 51, 48,
        67, 48, 49, 56, 54, 55, 48, 32, 32, 49, 48, 50, 48, 50, 50, 48, 32, 80, 114, 101, 101, 109, 112, 116, 105, 118,
        101, 32, 32, 48, 48, 48, 48, 55, 70, 67, 51, 51, 68, 68, 51, 65, 69, 49, 56, 58, 48, 48, 48, 48, 55, 70, 67, 51,
        51, 68, 68, 51, 67, 57, 65, 48, 32, 48, 48, 48, 48, 55, 70, 67, 68, 57, 70, 69, 52, 52, 48, 54, 48, 32, 45, 48,
        48, 48, 48, 49, 32, 85, 107, 110, 32, 40, 84, 104, 114, 101, 97, 100, 112, 111, 111, 108, 32, 87, 111, 114, 107,
        101, 114, 41, 32, 0
    };

    var s1 = "ThreadCount:      42";
    var s2 = " DBG   ID     OSID ThreadOBJ           State GC Mode     GC Alloc Context                  Domain           Count Apt Exception";
    var s3 = "  11    5       13 00007FC30C018670  1020220 Preemptive  00007FC33DD3AE18:00007FC33DD3C9A0 00007FCD9FE44060 -00001 Ukn (Threadpool Worker) ";

    foreach (var (x, s) in new[] { (x1, s1), (x2, s2) , (x3, s3) })
    {
        fixed (byte* ch = x)
        {
            var rs = Interop.call_regex(ref Unsafe.AsRef<sbyte>(ch));
            var cs = CSRegexBenchmark.CSharpRegex().IsMatch(s);

            Console.WriteLine($"RS: {rs} . CS: {cs}");
        }
    }
}

// Console.ReadLine();
// CallStr();

BenchmarkRunner.Run(Assembly.GetExecutingAssembly());