using System.Diagnostics;
using System.Threading.Channels;
using GameWatcher;
using Microsoft.Extensions.Hosting;
using Microsoft.Win32;

Process? TryGetProcessById(int id)
{
    try
    {
        return Process.GetProcessById(id);
    }
    catch
    {
        return null;
    }
}

Process? GetGameProcess(HashSet<string> games)
{
    var processes = Process.GetProcesses();

    foreach (var process in processes)
    {
        if (games.Contains(process.MainModule!.FileName))
        {
            return process;
        }
    }

    return null;
}

Process? GetGameProcessById(int id, HashSet<string> games)
{
   if (TryGetProcessById(id) is {} process
       && games.Contains(process.MainModule!.FileName))
    {
        return process;
    }

    return null;
}

var games =
    File.ReadAllLines(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "my.games"))
        .ToHashSet();

Console.WriteLine("Starting app with games loaded:");
foreach (var game in games)
{
    Console.WriteLine(game);
}

var channel = Channel.CreateUnbounded<int>(new() { SingleReader = true } );

_ = Task.Run(async () =>
{
    var currentGameProcessId = -1;
    var resume = false;
    
    await foreach (var _ in channel.Reader.ReadAllAsync())
    {
        if ((GetGameProcessById(currentGameProcessId, games)
             ?? GetGameProcess(games)) is {} process)
        {
            currentGameProcessId = process.Id;
            if (resume)
            {
                Console.WriteLine($"Resuming {process.Id} {process.MainModule!.FileName}");
                await process.Resume();
            }
            else
            {
                Console.WriteLine($"Suspending {process.Id} {process.MainModule!.FileName}");
                await process.Suspend();           
            }
        }
        
        resume = !resume;
    }
});

SystemEvents.PowerModeChanged += async (_, eventArgs) =>
{
    switch (eventArgs.Mode)
    {
        case PowerModes.Suspend:
            Console.WriteLine("System is suspended...");
            await channel.Writer.WriteAsync(0);
            break;

        case PowerModes.Resume:
            Console.WriteLine("System is resumed...");
            await channel.Writer.WriteAsync(0);
            break;
    }
};

var host = new HostBuilder().Start();
await host.WaitForShutdownAsync();