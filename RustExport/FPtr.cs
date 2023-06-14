using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace RustExport;

public unsafe class FPtr
{
    private static readonly nint LibAddress =
        NativeLibrary.Load(
            OperatingSystem.IsWindows()
                ? Interop.NativeLib
                : OperatingSystem.IsLinux()
                    ? $"./{Interop.NativeLib}.so"
                    : throw new($"OS not supported: {Environment.OSVersion}"));

    public static readonly delegate* unmanaged[Cdecl] <byte*, bool> call_regex =
        (delegate* unmanaged[Cdecl] <byte*, bool>) NativeLibrary.GetExport(LibAddress, nameof(call_regex));
}

public partial class LibraryImport
{
    [LibraryImport(Interop.NativeLib, EntryPoint = "call_regex")]
    [UnmanagedCallConv(CallConvs = new[] { typeof(CallConvCdecl) })]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool call_regex(ref sbyte ch);
}