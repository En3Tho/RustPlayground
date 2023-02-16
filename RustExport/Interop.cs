// Automatically generated by Interoptopus.

#pragma warning disable 0105
using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using RustExport;
#pragma warning restore 0105

namespace RustExport
{
    public static partial class Interop
    {
        public const string NativeLib = "rust_export";

        static Interop()
        {
        }


        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "my_function")]
        public static extern Vec2 my_function(Vec2 input);

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "call_regex")]
        public static extern bool call_regex(ref sbyte ch);

    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    public partial struct Vec2
    {
        public float x;
        public float y;
    }



    public class InteropException<T> : Exception
    {
        public T Error { get; private set; }

        public InteropException(T error): base($"Something went wrong: {error}")
        {
            Error = error;
        }
    }

}