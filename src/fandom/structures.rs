use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use Lang::*;
use Namespace::*;
use Wikis::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct RandomPage {
    pub(crate) title: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RandomQuery {
    pub(crate) random: Vec<RandomPage>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RandomRes {
    pub(crate) query: RandomQuery,
}

pub struct GenericPage {
    pub title: String,
    pub desc: Option<String>,
    pub link: String,
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<reqwest::Client>;
}

impl From<RandomPage> for GenericPage {
    fn from(page: RandomPage) -> GenericPage {
        GenericPage {
            title: page.title.clone(),
            desc: Some("Random page...".into()),
            link: format!(
                "{}/{}",
                LOTRMod(En).site(),
                page.title.trim().replace(" ", "_")
            ),
        }
    }
}

#[derive(std::cmp::PartialEq)]
pub enum Lang {
    En,
    Fr,
    De,
    Nl,
    Zh,
    Ru,
    Es,
    Ja,
}

impl Lang {
    pub(crate) fn main(&self) -> String {
        match self {
            En => "The Lord of the Rings Minecraft Mod Wiki",
            Fr => "Wiki du Mod Minecraft Seigneur des Anneaux",
            De => "Der Herr der Ringe Minecraft Mod Wiki",
            Nl => "In de ban van de Ring Minecraft Mod wiki",
            Zh => "魔戒我的世界模组百科",
            Ru => "Средиземье в Minecraft",
            Es => "Wiki Lotrminecraftmod",
            Ja => "マインクラフト　指輪物語MOD Wiki",
        }
        .into()
    }

    fn maindesc(&self, username: &str) -> String {
        match self {
            En => format!("Welcome, {}, to The Lord of the Rings Minecraft Mod Wiki, the official public wiki for everything related to the Lord of the Rings Mod.", username),
            Fr => format!("Bienvenue, {}, sur le Wiki du Mod Seigneur des Anneaux pour Minecraft, un wiki public pour tout ce qui concerne le Mod Seigneur des Anneaux.", username),
            De => format!("Willkommen, {}, im Der Herr der Ringe Minecraft Mod Wiki, einem öffentlichem Wiki für alles, was sich auf die Der Herr der Ringe Mod bezieht.", username),
            Nl => format!("Welkom, {}, op de In de ban van de Ring Minecraft Mod wiki, de officiële openbare Nederlandstalige wiki voor alles in verband met de In de ban van de Ring Mod.", username),
            Zh => "欢迎你来到魔戒我的世界模组百科！".into(),
            Ru => format!("Добро пожаловать, {}, на Вики, связанную с модом Lord of the Rings Mod.", username),
            Es => "Bienvenidos a Wiki Lotrminecraftmod\nEl wiki sobre el mod El Señor de los Anillos para Minecraft que todos pueden editar.".into(),
            Ja => "このサイトはThe Lord of The Rings Minecraft Mod Wiki、指輪物語MODに関する公式Wikiの日本語版です。FANDOMのアカウントを作成して言語設定を日本語にすることで、メニュー周りも日本語になり読みやすくなります。".into()
        }
    }

    fn users(&self) -> String {
        match self {
            En => "Users",
            Fr => "Liste des utilisateurs",
            De => "Benutzer",
            Nl => "Gebruikerslijst",
            Zh => "用户列表",
            Ru => "Список участников",
            Es => "Lista Usuarios",
            Ja => "登録利用者一覧",
        }
        .into()
    }

    fn files(&self) -> String {
        match self {
            En => "List Files",
            Fr => "Liste des fichiers",
            De => "Dateien",
            Nl => "Bestandenlijst",
            Zh => "文件列表",
            Ru => "Список файлов",
            Es => "Lista Imágenes",
            Ja => "ファイル一覧",
        }
        .into()
    }

    fn templates(&self) -> String {
        match self {
            En => "Templates",
            Fr => "Modèles",
            De => "Vorlagen",
            Nl => "Sjablonen",
            Zh => "Templates",
            Ru => "Шаблоны",
            Es => "Plantillas",
            Ja => "テンプレート",
        }
        .into()
    }

    fn categories(&self) -> String {
        match self {
            En => "Categories",
            Fr => "Catégories",
            De => "Kategorien",
            Nl => "Categorieën",
            Zh => "页面分类",
            Ru => "Категории",
            Es => "Categorías",
            Ja => "カテゴリ",
        }
        .into()
    }

    fn blogs(&self) -> String {
        match self {
            En => "Recent posts",
            Fr => "Posts récents",
            De => "Letzte Beiträge",
            Nl => "Recente berichten",
            Zh => "最近的职位",
            Ru => "Последние сообщения",
            Es => "Entradas recientes",
            Ja => "最近の投稿",
        }
        .into()
    }
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            En => write!(f, "wiki"),
            Fr => write!(f, "fr"),
            De => write!(f, "de"),
            Nl => write!(f, "nl"),
            Zh => write!(f, "zh"),
            Ru => write!(f, "ru"),
            Es => write!(f, "es"),
            Ja => write!(f, "ja"),
        }
    }
}

#[derive(std::cmp::PartialEq)]
pub enum Wikis {
    LOTRMod(Lang),
    TolkienGateway,
    Minecraft,
}

impl Wikis {
    pub(crate) fn get_api(&self) -> String {
        match self {
            LOTRMod(En) => "https://lotrminecraftmod.fandom.com/api.php?".to_string(),
            LOTRMod(lang) => format!("https://lotrminecraftmod.fandom.com/{}/api.php?", lang),
            TolkienGateway => "http://tolkiengateway.net/w/api.php?".to_string(),
            Minecraft => "https://minecraft.gamepedia.com/api.php?".to_string(),
        }
    }

    pub fn site(&self) -> String {
        match self {
            LOTRMod(lang) => format!("https://lotrminecraftmod.fandom.com/{}", lang),
            TolkienGateway => "https://tolkiengateway.net".to_string(),
            Minecraft => "https://minecraft.gamepedia.com".to_string(),
        }
    }

    pub fn default_img(&self) -> String {
        match self {
            LOTRMod(_) => {
                "https://static.wikia.nocookie.net/lotrminecraftmod/images/8/8e/GrukRenewedLogo.png"
            }
            TolkienGateway => "https://medias.liberation.fr/photo/1277413-author-j-r-r-tolkien.jpg",
            Minecraft => "https://i.ytimg.com/vi/Zeh9lmHGVM4/maxresdefault.jpg",
        }
        .into()
    }

    pub fn icon(&self) -> String {
        match self {
            LOTRMod(_) => "https://i.ibb.co/v1hHg3G/test.png",
            TolkienGateway => "https://i.ibb.co/VYKWK7V/favicon.png",
            Minecraft => {
                "https://toppng.com/uploads/preview/minecraft-block-icon-11531077309p00lhxolea.png"
            }
        }
        .to_string()
    }

    pub fn name(&self) -> String {
        match self {
            LOTRMod(lang) => lang.main(),
            TolkienGateway => "Tolkien Gateway".into(),
            Minecraft => "Official Minecraft Wiki".into(),
        }
    }

    pub fn default(&self, username: &str) -> GenericPage {
        match self {
            LOTRMod(_) => Page.main_page(self, username),
            TolkienGateway => GenericPage {
                title: self.name(),
                link: self.site(),
                desc: Some(format!(
                    "Welcome, {}, to Tolkien Gateway,
the J.R.R. Tolkien encyclopedia that anyone can edit.",
                    username
                )),
            },
            Minecraft => GenericPage {
                title: self.name(),
                link: self.site(),
                desc: Some(format!(
                    "Welcome, {}, to the Official Minecraft Wiki,
a publicly accessible and editable wiki for information on Minecraft and related subjects.",
                    username
                )),
            },
        }
    }
}

#[derive(std::cmp::PartialEq)]
pub enum Namespace {
    Page,
    User,
    File,
    Template,
    Category,
    Blog,
}

impl From<&Namespace> for String {
    fn from(namespace: &Namespace) -> String {
        match namespace {
            Page => "0|4",
            User => "2",
            File => "6",
            Template => "10",
            Category => "14",
            Blog => "500",
        }
        .into()
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page => write!(f, "page"),
            User => write!(f, "user"),
            File => write!(f, "file"),
            Template => write!(f, "template"),
            Category => write!(f, "category"),
            Blog => write!(f, "blog post"),
        }
    }
}

impl Namespace {
    pub fn main_page(&self, wiki: &Wikis, username: &str) -> GenericPage {
        match wiki {
            LOTRMod(lang) => {
                match self {
                    Page => GenericPage {
                        title: lang.main(),
                        link: wiki.site(),
                        desc: Some(lang.maindesc(username)),
                    },
                    User => GenericPage {
                        title: lang.users(),
                        link: format!("{}/Special:Listusers", wiki.site()),
                        desc: None,
                    },
                    File => GenericPage {
                        title: lang.files(),
                        link: format!("{}/Special:ListFiles", wiki.site()),
                        desc: None,
                    },
                    Template => GenericPage {
                        title: lang.templates(),
                        link: format!("{}/Special:PrefixIndex?namespace=10", wiki.site()),
                        desc: None,
                    },
                    Category => GenericPage {
                        title: lang.categories(),
                        link: format!("{}/Special:Categories", wiki.site()),
                        desc: None,
                    },
                    Blog => GenericPage {
                        title: lang.blogs(),
                        link: format!("{}/Blog:Recent_posts", wiki.site()),
                        desc: None,
                    },
                }
            },
            TolkienGateway => GenericPage {
                title: "Tolkien Gateway".into(),
                link: wiki.site(),
                desc: Some("Welcome to Tolkien Gateway,\nthe J.R.R. Tolkien encyclopedia that anyone can edit.".into()),
            },
            Minecraft => GenericPage {
                title: "Official Minecraft Wiki".into(),
                link: wiki.site(),
                desc: Some("".into()),
            }
        }
    }
}
