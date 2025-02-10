using System.Diagnostics;

namespace GameWatcher;

public static class ProcessExtensions
{
    private static Task ChangeProcessState(Process process, bool resume)
    {
        var resumeTag = resume ? " -r" : "";
        var procStartInfo = new ProcessStartInfo("pssuspend64")
        {
            Arguments = $"{process.Id}{resumeTag}"
        };
        return Process.Start(procStartInfo)!.WaitForExitAsync();
    }

    public static Task Suspend(this Process process) => ChangeProcessState(process, false);
    public static Task Resume(this Process process) => ChangeProcessState(process, true);
}