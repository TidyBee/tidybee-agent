namespace api.Models {

    public class TidyScoreModel
    {
        public bool Misnamed { get; set; }
        public bool Misplaced { get; set; }
        public bool Unused { get; set; }
        public FileModel Duplicated { get; set; }
    }
}
