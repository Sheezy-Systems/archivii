class Post:
    def __init__(self, authorID, authorName, postID, likeCount, content):
        self.author = {"Name": authorName, "id": authorID}
        self.content = content
        self.likeCount = likeCount
        self.id = postID

    def __str__(self) -> str:
        return f"{self.author['Name']} ({self.author['id']}) - {self.content} - {self.likeCount} likes"