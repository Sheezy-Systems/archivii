import json
import time
import requests
import bs4
import re
import codecs
import os
from dotenv import load_dotenv
import colorama
settings = {}
posts, tmp = [], []
previousCount = 0

class Post:
    def __init__(self, authorID, authorName, postID, likeCount, content):
        self.author = {"Name": authorName, "id": authorID}
        self.content = content
        self.likeCount = likeCount
        self.id = postID

    def __str__(self):
        return f'{self.author}: {self.content}'

def do_request(reqURL):
    cookies = {
        settings.get("COOKIE_NAME"): os.environ["SECRET"] # Authorization cookie has this name
    }
    response = requests.get(reqURL, cookies=cookies)
    time.sleep(0.1) # Schoology has a rate limit of 3 requests per second, this works for some reason
    return response

def parseLink(TYPE, GROUP_ID, page=0):
    global previousCount, tmp
    url = settings.get("BASE_URL") + "/" + TYPE + "/" + GROUP_ID + '/feed?page=' + str(page)
    print("Checking page " + str(page) + "...")
    authorID, postID, likeCount, = "" , "", 0
    response = do_request(url)
    html = json.loads(response.text).get('output') # get html from json
    html = re.subn(r'<(script).*?</\1>(?s)', '', html, flags=re.DOTALL)[0] # remvoe js from html
    soup = bs4.BeautifulSoup(html, 'html.parser') # use bs4 to parse html
    
    #write beautified version
    with codecs.open("out.html", 'w', "utf-8") as f:
        f.write(str(soup.prettify()))
    
    for post in soup.find_all(class_='s-edge-type-update-post'):
        authorelement = post.find(class_='update-sentence-inner').find('a')
        text = ""
        for i in range(len(post.find_all('p'))):
            text += post.find_all('p')[i].text + '\n' # add newline to end of each paragraph
        text = text[:-2] # remove trailing newline
        for link in post.find_all("a"):
            splitURL = link.get("href").split("/")
            linkClass = link.get("class") # get class name of link
            # if linkClass == ['like-details-btn'] or "like-details-btn":
            #     authorID = splitURL[-1]
            if str(link.get("id")).endswith("-show-more-link") and (linkClass == ['show-more-link'] or linkClass == 'show-more-link'): # Has a show more button; all text not being shown
                text = fetchFullText(splitURL[2], splitURL[-1])
            try:
                if splitURL[2] == "n" and (linkClass == ['like-details-btn'] or linkClass =='like-details-btn'):
                    likeCount = link.text
                    postID = splitURL[-1]
            except IndexError:
                pass # Not a like count anyways
            text = text.replace("\u00a0", "")
            text = text.replace("\\n", "\n")

        tmp.append(Post(authorelement.get("href").split("/")[-1], authorelement.text, postID, likeCount, text))

    if len(tmp) > previousCount and len(tmp) != 0:
        previousCount = len(tmp)
        page += 1
        parseLink(TYPE, GROUP_ID, page)
    return tmp

def fetchFullText(ID1, ID2):
    url = settings.get("BASE_URL") + "/update_post/" + ID1 + "/show_more/" + ID2
    response = do_request(url)
    try:
        soup = bs4.BeautifulSoup(json.loads(response.text).get("update"), 'html.parser')
    except:
        print("Error fetching full text")
        print(response.text)
        return ""
    return(soup.text)

if __name__ == '__main__':
    try:
        with open('config.json') as f:
            settings = json.load(f)
    except [FileNotFoundError]:
        print("No config file found")
        exit()

    load_dotenv()
    posts = parseLink(settings.get("TYPE"), settings.get("ID"))
    print("Done, found " + str(len(posts)) + " posts.")
    with open("posts.json", 'w') as f:
        f.write(json.dumps([ob.__dict__ for ob in posts], indent=4))