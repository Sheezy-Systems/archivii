import json
import requests
import bs4
import re
import codecs
import os
posts = []

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

def parseLink(url):
    tmp = []
    response = do_request(url, os.environ["SECRET"])
    html = json.loads(response.text).get('output')
    html = re.subn(r'<(script).*?</\1>(?s)', '', html, flags=re.DOTALL)[0]
    soup = bs4.BeautifulSoup(html, 'html.parser')

    
    with codecs.open("out.html", 'w', "utf-8") as f:
        f.write(str(soup.prettify()))
    
    for post in soup.find_all(class_='s-edge-type-update-post'):
        author = post.find(class_='update-sentence-inner').find('a').text
        text = ""
        for i in range(len(post.find_all('p'))):
            text += post.find_all('p')[i].text + '\n'
        text = text[:-2]
        tmp.append(Post(None, author, text))

    return tmp



if __name__ == '__main__':
    BASE_URL = "https://schoology.tesd.net/group/"
    GROUP_ID = "812485279"
    url = BASE_URL + GROUP_ID + '/feed?page=0'
    parseLink(url)
