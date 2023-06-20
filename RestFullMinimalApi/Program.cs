using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
// Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();



builder.Services.AddDbContext<TodoDb>(opt =>
    opt.UseInMemoryDatabase("TodoList"));
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
    var files = Directory.GetFiles(@"C:\Users\guill\Downloads");
    return files;
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
    Directory.Delete(@"C:\Users\guill\Downloads\EIPPPP");
    return "Directory Deleted";
});


app.MapGet("/getFiles/{path}", (string path) =>
{
    var files = Directory.GetFiles(path);
    Console.WriteLine(files);
    return files;
});

app.MapGet("/getDirectories/{path}", (string path) =>
{
    var dirs = Directory.GetDirectories(path);
    return dirs;
});

app.MapPost("/postDirectory/{path}", (string path) =>
{
    //string directoryPath = Path.Combine(path, "EIPPPP");
    Directory.CreateDirectory(path);
    return "Directory Created";
});

app.MapDelete("/deleteDirectory/{path}", (string path) =>
{
    //string directoryPath = Path.Combine(path, "EIPPPP");
    Directory.Delete(path);
    return "Directory Deleted";
});



app.MapGet("/todoitems", async (TodoDb db) =>
    await db.Todos.ToListAsync());
app.MapGet("/todoitems/complete", async (TodoDb db) =>
    await db.Todos.Where(t => t.IsComplete).ToListAsync());
app.MapGet("/todoitems/{id}", async (int id, TodoDb db) =>
    await db.Todos.FindAsync(id)
        is Todo todo
        ? Results.Ok(todo)
        : Results.NotFound());
app.MapPost("/todoitems", async (Todo todo, TodoDb db) =>{
    db.Todos.Add(todo);
    await db.SaveChangesAsync();
    return Results.Created($"/todoitems/{todo.Id}", todo);
});
app.MapPut("/todoitems/{id}", async (int id, Todo inputTodo, TodoDb db) =>
{
    var todo = await db.Todos.FindAsync(id);
    if (todo is null) return Results.NotFound();
    todo.Name = inputTodo.Name;
    todo.IsComplete = inputTodo.IsComplete;
    await db.SaveChangesAsync();
    return Results.NoContent();
});
app.MapDelete("/todoitems/{id}", async (int id, TodoDb db) =>
{
    if (await db.Todos.FindAsync(id) is Todo todo)
    {
        db.Todos.Remove(todo);
        await db.SaveChangesAsync();
        return Results.Ok(todo);
    }
    return Results.NotFound();
});
app.Run();
class Todo
{
    public int Id { get; set; }
    public string? Name { get; set; }
    public bool IsComplete { get; set; }
}

class TodoDb : DbContext
{
    public TodoDb(DbContextOptions<TodoDb> options)
        : base(options)
    {
    }

    public DbSet<Todo> Todos => Set<Todo>();
}