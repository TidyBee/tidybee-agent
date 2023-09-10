namespace api.Models {

    public class FileModel
    {
        public string Path { get; set; }
        public uint Size { get; set; }
        public string LastAccess { get; set; }
        public TidyScoreModel TidyScore { get; set; }
    }
}