class Post:
    def __init__(self, authorID, authorName, postID, likeCount, content, comments):
        self.author = {"name": authorName, "id": authorID}
        self.content = content
        self.likeCount = likeCount
        self.id = postID
        self.comments = comments

    def __str__(self) -> str:
        return f"{self.author['name']} ({self.author['id']}) - {self.content} - {self.likeCount} likes"