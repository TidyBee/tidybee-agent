using Microsoft.AspNetCore.Mvc;
using System;
using System.Collections.Generic;
using api.Models;

namespace api.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class DashboardController : ControllerBase
    {
        [HttpGet("top-heaviest-files")]
        public ActionResult<IEnumerable<FileModel>> GetTopHeaviestFiles()
        {
            var fakeData = new List<FileModel>
            {
                new FileModel
                {
                    Path = "file1.txt",
                    Size = 1024,
                    LastAccess = DateTime.Now.ToString("yyyy-MM-dd"),
                    TidyScore = new TidyScoreModel
                    {
                        Misnamed = false,
                        Misplaced = false,
                        Unused = false,
                        Duplicated = null
                    }
                },
                new FileModel
                {
                    Path = "file2.txt",
                    Size = 44535190,
                    LastAccess = DateTime.Now.ToString("yyyy-MM-dd"),
                    TidyScore = new TidyScoreModel
                    {
                        Misnamed = true,
                        Misplaced = false,
                        Unused = true,
                        Duplicated = null
                    }
                },
            };
            return Ok(fakeData);
        }
    }
}
