import json
from post import Post
posts = {}

with open('posts.json', 'r') as f:
    posts = json.load(f)    

posts.sort(key=lambda x: x['likeCount'], reverse=True)

for i in range(len(posts)):
    posts[i] = Post(posts[i]['author']['id'], posts[i]['author']['Name'], posts[i]['id'], posts[i]['likeCount'], posts[i]['content'])

#for each author, get the mean number of likes on their posts
authors = {}
for post in posts:
    if post.author.get("Name") in authors:
        authors[post.author.get("Name")].append(post.likeCount)
    else:
        authors[post.author.get("Name")] = [post.likeCount]

for author in authors:
    print(""+author + ': ' + str(round(sum(authors[author])/len(authors[author]), 3)) + ' averages likes per post')

with open("posts.json", 'w') as f:
    f.write(json.dumps([ob.__dict__ for ob in posts], indent=4))

