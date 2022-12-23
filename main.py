import json
import time
import requests
import bs4
import re
import codecs
from codecs import encode
import os
from dotenv import load_dotenv
import colorama
from post import Post
settings = {}
posts, tmp = [], []
previousCount = 0
SECRET = None
RATELIMIT = settings.get("RATELIMIT", 330)/1000

def do_request(reqURL):
    cookies = {
        settings.get("COOKIE_NAME"): SECRET # Authorization cookie has this name
    }
    headers = {
       "accept": "application/json, text/javascript, */*; q=0.01",
    }
    response = requests.get(reqURL, cookies=cookies, headers=headers)
    response.encoding = 'ascii'
    time.sleep(RATELIMIT) # Schoology has a rate limit of 3 requests per second, this works for some reason
    return response

def parseLink(TYPE, REALM_ID, page=0, limit=1000):
    global previousCount, tmp
    url = settings.get("BASE_URL") + "/" + TYPE + "/" + REALM_ID + '/feed?page=' + str(page)
    print("Checking page " + str(page) + "...")
    authorID, postID, likeCount, = "" , "", 0
    response = do_request(url)
    try:
        html = json.loads(response.text.encode('utf-8').decode("ascii")).get('output') # get html from json
    except:
        print(colorama.Fore.RED + "Error fetching page" + colorama.Style.RESET_ALL)
        print(response.text)
        exit()
    html = removeScripts(html)
    soup = bs4.BeautifulSoup(html, 'html.parser') # use bs4 to parse html
    
    #write beautified version
    with codecs.open("out.html", 'w', "utf-8") as f:
        f.write(str(soup.prettify()))
    
    for post in soup.find_all(class_='s-edge-type-update-post'):
        authorelement = post.find(class_='update-sentence-inner').find('a')
        commentList = []
        text = ""
        likeBtn = post.find(class_='like-btn')
        postID = likeBtn.get("ajax").split("/")[-1]
        for i in range(len(post.find_all('p'))):
            text += post.find_all('p')[i].text + '\n' # add newline to end of each paragraph
        text = text[:-2] # remove trailing newline
        for link in post.find_all("a"):
            splitURL = link.get("href").split("/")
            linkClass = link.get("class") # get class name of link
            if str(link.get("id")).endswith("-show-more-link") and (linkClass == ['show-more-link'] or linkClass == 'show-more-link'): # Has a show more button; all text not being shown
                text = fetchFullText(splitURL[2], splitURL[-1])
            try:
                if splitURL[2] == "n" and (linkClass == ['like-details-btn'] or linkClass =='like-details-btn'):
                    likeCount = int(link.text.split(" ")[0])
            except IndexError:
                pass # Not a like count anyways
            text = text.replace("\u00a0", "")
            text = text.replace("\\n", "\n")
        comments = soup.find_all('div', class_='comment-comment')
        for comment in comments:
            if comment.find_parent('div', class_='s-edge-type-update-post') == post:
                if len(comments) >= 3:
                    # Schoology only shows 3 comments, so we need to fetch the rest
                    commentList = fetchAllComments(postID)
                    break
                else:
                    commentList = parseComment(comment)

        tmp.append(Post(authorelement.get("href").split("/")[-1], authorelement.text, postID, likeCount, text, commentList))

    if len(tmp) > previousCount and len(tmp) != 0:
        if page + 1 > limit:
            return tmp
        previousCount = len(tmp)
        page += 1
        parseLink(TYPE, REALM_ID, page, limit)
    return tmp

def fetchFullText(ID1, ID2):
    url = settings.get("BASE_URL") + "/update_post/" + ID1 + "/show_more/" + ID2
    response = do_request(url)
    try:
        soup = bs4.BeautifulSoup(json.loads(response.text).get("update").encode().decode('unicode-escape'), 'html.parser')
    except:
        print(colorama.Fore.RED + "Error fetching full text" + colorama.Style.RESET_ALL)
        exit()
        return ""
    return(soup.text)

def fetchAllComments(postID):
    url = settings.get("BASE_URL") + "/comment/ajax/" + postID + "&context=updates"
    response = do_request(url)
    expression = re.compile("u'2022'", re.UNICODE)
    decoded = expression.sub("u'u2022'", response.text)
    return parseComment(removeScripts(decoded))

    

def parseComment(text):
    commentsList = []
    try:
        parsed = bs4.BeautifulSoup(json.loads(str(text)).get("comments"), 'html.parser')
    except:
        parsed = bs4.BeautifulSoup(str(text), 'html.parser')
        return []
    comments = parsed.find_all('div', class_='comment-comment')
    for comment in comments:
        author = comment.find(class_='comment-author').find('a')
        commentText = comment.find(class_='comment-body-wrapper')
        commentsList.append({
            "text": commentText.text,
            "author": {
                "name": author.text,
                "id": author.get("href").split("/")[-1]
            }
        })
    return commentsList

def removeScripts(html):
    return re.subn(r'(?s)<(script).*?</\1>', '', html, flags=re.DOTALL)[0]

if __name__ == '__main__':
    try:
        with open('config.json') as f:
            settings = json.load(f)
    except [FileNotFoundError]:
        print("No config file found")
        exit()
    load_dotenv()
    limit = settings.get("PAGE_LIMIT", 0x0FFFFFFF)
    if limit == None:
        limit = 0x0FFFFFFFF
    START = settings.get("START_PAGE", 0)
    if START == None:
        START = 0
    
    try:
        SECRET = os.environ["SECRET"]
    except KeyError:
        print("Could not find SECRET in your .env file")
        exit()

    posts = parseLink(settings.get("REALM"), settings.get("ID"), START, limit)
    print("Done, found " + str(len(posts)) + " posts.")
    with open("posts.json", 'w') as f:
        f.write(json.dumps([ob.__dict__ for ob in posts], indent=4))