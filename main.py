import json
import requests
import bs4
import re
import codecs
import os
from dotenv import load_dotenv
posts = []
BASE_URL, Type, GROUP_ID = None, None, None

class Post:
    def __init__(self, postID, author, content):
        self.author = author
        self.content = content
        self.postID = postID

    def __str__(self):
        return f'{self.author}: {self.content}'

def do_request(reqURL, secret):
    cookies = {
        'SESS61c75f44be1e14cdb172294ad6a89a4e': secret # Authorization cookie has this name
    }
    response = requests.get(reqURL, cookies=cookies)
    return response

def parseLink(TYPE, GROUP_ID):
    global BASE_URL
    url = BASE_URL + "/" + TYPE + "/" + GROUP_ID + '/feed?page=0'
    print(url)
    tmp = []
    authorID = None
    response = do_request(url, os.environ["SECRET"])
    html = json.loads(response.text).get('output')
    html = re.subn(r'<(script).*?</\1>(?s)', '', html, flags=re.DOTALL)[0]
    soup = bs4.BeautifulSoup(html, 'html.parser')
    
    #write beautified version
    with codecs.open("out.html", 'w', "utf-8") as f:
        f.write(str(soup.prettify()))
    
    for post in soup.find_all(class_='s-edge-type-update-post'):
        author = post.find(class_='update-sentence-inner').find('a').text
        text = ""
        for i in range(len(post.find_all('p'))):
            text += post.find_all('p')[i].text + '\n'
        text = text[:-2] # remove trailing newline
        for link in post.find_all("a"):
            linkClass = link.get("class")
            print(linkClass)
            if linkClass == ['show-more-link'] or 'show-more-link': # Has a show more button; all text not being shown
                print("Show more link found")
                text += "..."
                fetchFullText(link.get("href").split("/")[-1])
                break
            elif linkClass == ['like-details-btn'] or "like-details-btn":
                authorID = link.get("href").split("/")[-1]
        
        tmp.append(Post(authorID, author, text))
    return tmp

def fetchFullText(postID):
    pass

if __name__ == '__main__':
    BASE_URL = "https://schoology.tesd.net"
    TYPE = "group"
    GROUP_ID = "812485279"
    load_dotenv()
    posts = parseLink(TYPE, GROUP_ID)

    for post in posts:
        print (str(post) + "\n\n")
    print("Found " + str(len(posts)) + " posts")