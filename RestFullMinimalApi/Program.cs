using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
// Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();

var app = builder.Build();
// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI();
}


app.MapGet("/", () => "Hello World!");

app.MapGet("/getFiles", () =>
{
    var fs = new List<File> { };
    for(int i = 0; i < 10; i++)
    {
        var f = new File(Name: "test" + i, Path: "test" + i, Extension: "test" + i, CreationTime: "test" + i, LastAccessTime: "test" + i, LastWriteTime: "test" + i, Size: "test" + i, Id: i, IsComplete: false);
        fs.Add(f);
    }
    //var files = Directory.GetFiles(@"C:\Users\guill\Downloads");
    return fs;
});


app.MapGet("/getDirectories", () =>
{
    var dirs = Directory.GetDirectories(@"C:\Users\guill\Downloads");
    return dirs;
});

app.MapPost("/postDirectory", () =>
{
    Directory.CreateDirectory(@"C:\Users\guill\Downloads\EIPPPP");
    return "Directory Created";
}); 

app.MapDelete("/deleteDirectory", () =>
{
    if (Directory.Exists(@"C:\Users\guill\Downloads\EIPPPP")) { 
        Directory.Delete(@"C:\Users\guill\Downloads\EIPPPP");
        return "Directory Deleted";
     }
    return "Directory does not exist";
});


app.MapGet("/getFiles/{path}", (string path) =>
{
    if (Directory.Exists(path))
    {
        var files = Directory.GetFiles(path);
        return files;
    }
    return null;
});

app.MapGet("/getDirectories/{path}", (string path) =>
{
    var dirs = Directory.GetDirectories(path);
    return dirs;
});

app.MapPost("/postDirectory/{path}", (string path) =>
{
    Directory.CreateDirectory(path);
    return "Directory Created";
});

app.MapDelete("/deleteDirectory/{path}", (string path) =>
{
    if (Directory.Exists(path))
    {
        Directory.Delete(path);
        return "Directory Deleted";
    }
    return "Directory does not exist";
});

app.Run();

class File
{
    public File(string Name, string Path, string Extension, string CreationTime, string LastAccessTime, string LastWriteTime, string Size, int Id, bool IsComplete)
    {
        this.Name = Name;
        this.Path = Path;
        this.Extension = Extension;
        this.CreationTime = CreationTime;
        this.LastAccessTime = LastAccessTime;
        this.LastWriteTime = LastWriteTime;
        this.Size = Size;
        this.Id = Id;
        this.IsComplete = IsComplete;
    }

    public string Name { get; set; }
    public string Path { get; set; }
    public string Extension { get; set; }
    public string CreationTime { get; set; }
    public string LastAccessTime { get; set; }
    public string LastWriteTime { get; set; }
    public string Size { get; set; }
    public int Id { get; set; }
    public bool IsComplete { get; set; }
}