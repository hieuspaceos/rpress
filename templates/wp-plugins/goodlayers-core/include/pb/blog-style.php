<?php
	/*	
	*	Goodlayers Blog Item Style
	*/
	
	if( !class_exists('gdlr_core_blog_style') ){
		class gdlr_core_blog_style{

			public $blog_item_id;
			public $blog_info_prefix = array();

			function __construct(){

				global $blog_item_id;
				$blog_item_id = empty($blog_item_id)? 1: $blog_item_id+1;
				$this->blog_item_id = $blog_item_id;

				$this->blog_info_prefix = apply_filters('gdlr_core_blog_info_prefix', array(
					'date' => '<i class="icon_clock_alt" ></i>',
					'tag' => '<i class="icon_tags_alt" ></i>',
					'category' => '<i class="icon_folder-alt" ></i>',
					'comment' => '<i class="icon_comment_alt" ></i>',
					'like' => '<i class="icon_heart_alt" ></i>',
					'author' => '<i class="icon_documents_alt" ></i>',
					'comment-number' => '<i class="icon_comment_alt" ></i>',
				));

			}

			// set blog info prefix
			function get_blog_info_prefix(){
				return $this->blog_info_prefix;
			}
			function set_blog_info_prefix($blog_info_prefix){
				$this->blog_info_prefix =  $blog_info_prefix;
			}

			// get the content of the blog item
			function get_content( $args ){

				$ret = apply_filters('gdlr_core_blog_style_content', '', $args, $this);
				if( !empty($ret) ) return $ret;

				switch( $args['blog-style'] ){
					case 'blog-full': 
					case 'blog-full-with-frame': 
						if( empty($args['blog-full-style']) || $args['blog-full-style'] == 'style-1' ){
							return $this->blog_full( $args ); 
						}else if( $args['blog-full-style'] == 'style-2' ){
							return $this->blog_full_style_2( $args ); 
						}else if( $args['blog-full-style'] == 'style-2-date' ){
							return $this->blog_full_style_2_date( $args ); 
						}else if( $args['blog-full-style'] == 'style-2-tag' ){
							return $this->blog_full_style_2_tag( $args ); 
						}else if( $args['blog-full-style'] == 'style-3' ){
							return $this->blog_full_style_3( $args ); 
						}
						break;
					case 'blog-column': 
					case 'blog-column-no-space': 
					case 'blog-column-with-frame': 
						if( empty($args['blog-column-style']) || $args['blog-column-style'] == 'style-1' ){
							return $this->blog_grid( $args );
						}else if( $args['blog-column-style'] == 'style-2' ){
							return $this->blog_grid_style_2( $args );
						}else if( $args['blog-column-style'] == 'style-2-date' ){
							return $this->blog_grid_style_2_date( $args );
						}else if( $args['blog-column-style'] == 'style-2-tag' ){
							return $this->blog_grid_style_2_tag( $args );
						}else if( $args['blog-column-style'] == 'style-3' ){
							return $this->blog_grid_style_3( $args );
						}else if( $args['blog-column-style'] == 'style-3-date' ){
							return $this->blog_grid_style_3_date( $args );
						}else if( $args['blog-column-style'] == 'style-4' ){
							return $this->blog_grid_style_4( $args );
						}else if( $args['blog-column-style'] == 'style-4-left-button' ){
							$args['blog-column-style'] = 'style-4';
							$args['additional-class'] = 'gdlr-core-left-button '; 
							return $this->blog_grid_style_4( $args );
						}
						break;
					case 'blog-column-hover-bg':
						return $this->blog_column_hover_bg( $args );
						break;
					case 'blog-image':
					case 'blog-image-no-space': 
						return $this->blog_modern( $args ); 
						break;	
					case 'blog-feature':
						return $this->blog_feature( $args ); 
						break;	
					case 'blog-metro':
					case 'blog-metro-no-space': 
						return $this->blog_metro( $args ); 
						break;	
					case 'blog-left-thumbnail': 
					case 'blog-right-thumbnail':
						if( empty($args['blog-side-thumbnail-style']) || in_array($args['blog-side-thumbnail-style'], array('style-1', 'style-1-large')) ){
							return $this->blog_medium( $args ); 
						}else if( in_array($args['blog-side-thumbnail-style'], array('style-2', 'style-2-large')) ){
							return $this->blog_medium_style_2( $args ); 
						}
						break;
					case 'blog-widget': 
						return $this->blog_widget( $args ); 
					case 'blog-widget-feature': 
						return $this->blog_widget( $args, true ); 
						break;
					case 'blog-list': 
					case 'blog-list-center': 
						return $this->blog_list( $args ); 
						break;
				}
				
			}
			
			// get blog excerpt from $args
			function get_blog_excerpt( $args ){

				$ret = '';

				if( $args['excerpt'] == 'specify-number' ){
					if( !empty($args['excerpt-number']) || ( !empty($args['show-read-more']) && $args['show-read-more'] != 'none') ){
						$ret .= '<div class="gdlr-core-blog-content clearfix" >';
						if( !empty($args['excerpt-number']) ){
							$ret .= $this->blog_excerpt($args['excerpt-number']);
							$ret .= '<div class="clear"></div>';
						}
						$ret .= (empty($args['after-content']))? '': $args['after-content'];
						
						if( !empty($args['show-read-more']) && $args['show-read-more'] != 'none' ){
							$blog_read_more = '';
							if( $args['show-read-more'] == 'enable' ){
								$blog_read_more = apply_filters('gdlr_core_blog_read_more', '');
							}
							if( empty($blog_read_more) ){
								$blog_read_more = $this->blog_excerpt_read_more_button($args);
							}
							
							$ret .= $blog_read_more;
						}
						$ret .= '</div>';
					}
				}else if( $args['excerpt'] != 'none' ){
					$ret .= '<div class="gdlr-core-blog-content" >';
					$ret .= gdlr_core_content_filter(get_the_content(), true);
					$ret .= (empty($args['after-content']))? '': $args['after-content'];
					$ret .= '</div>';
				}

				return $ret;
			}
			function blog_excerpt_read_more_button($args){
				$blog_read_more = '';

				if( $args['show-read-more'] == 'enable' || $args['show-read-more'] == 'button' ||  $args['show-read-more'] == 'border-button' ){
					$blog_read_more .= '<a class="gdlr-core-excerpt-read-more gdlr-core-button gdlr-core-rectangle';
					if( $args['show-read-more'] == 'border-button' ){
						$blog_read_more .= ' gdlr-core-button-with-border gdlr-core-button-transparent';
					}
					$blog_read_more .= '" href="' . get_permalink() . '" >';
					if( !empty($args['read-more-button-text']) ){
						$blog_read_more .= gdlr_core_text_filter($args['read-more-button-text']);
					}else{
						$blog_read_more .= esc_html__('Read More', 'goodlayers-core');
					}
					$blog_read_more .= '</a>';
				}else if( $args['show-read-more'] == 'text' || $args['show-read-more'] == 'text-hover-border' ){
					$blog_read_more .= '<a class="gdlr-core-excerpt-read-more gdlr-core-plain-text ';
					$blog_read_more .= ($args['show-read-more'] == 'text-hover-border')? 'gdlr-core-hover-border': '';
					$blog_read_more .= '" href="' . get_permalink() . '" >';
					if( !empty($args['read-more-button-text']) ){
						$blog_read_more .= gdlr_core_text_filter($args['read-more-button-text']);
					}else{
						$blog_read_more .= esc_html__('Read More', 'goodlayers-core');
					}
					$blog_read_more .= '<i class="fa fa-long-arrow-right" ></i>';
					$blog_read_more .= '</a>';
				}

				return $blog_read_more;
			}

			// get blog excerpt
			function blog_excerpt( $excerpt_length ) {
				
				$post = get_post();

				if( empty($post) || post_password_required() ){ return ''; }
			
				$excerpt = $post->post_excerpt;
				if( empty($excerpt) ){
					$excerpt = get_the_content('');
					$excerpt = strip_shortcodes($excerpt);
					
					$excerpt = apply_filters('the_content', $excerpt);
					$excerpt = str_replace(']]>', ']]&gt;', $excerpt);
				}
				
				$excerpt_more = apply_filters('excerpt_more', '...');
				$excerpt = apply_filters('wp_trim_excerpt', $excerpt, $post->post_excerpt);		
				$excerpt = apply_filters('get_the_excerpt', $excerpt, $post);
				$excerpt = wp_trim_words($excerpt, $excerpt_length, $excerpt_more);
				
				
				return $excerpt;
			}			
			
			// get blog thumbnail
			function blog_thumbnail( $args = array(), $thumbnail_atts = array() ){

				$ret = '';

				if( !empty($args['post-format']) ){
					global $pages;

					// strip the media based on post format
					if( $args['post-format'] == 'video' ){
						if( !preg_match('#^https?://\S+#', $pages[0], $match) ){
							if( !preg_match('#^\[video\s.+\[/video\]#', $pages[0], $match) ){
								if( !preg_match('#^\[embed.+\[/embed\]#', $pages[0], $match) ){
									preg_match('#\<figure.+\<\/figure\>#sim', $pages[0], $match_html);
								}
							}
						}

						if( !empty($match[0]) ){
							if( isset($args['post-format-thumbnail']) && $args['post-format-thumbnail'] === false ){
								$thumbnail_size = 'full';
							}else{
								$thumbnail_size = $args['thumbnail-size'];
							}

							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-video" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
							$ret .= gdlr_core_get_video($match[0], $thumbnail_size);
							$ret .= '</div>';

							$pages[0] = substr($pages[0], strlen($match[0]));
						}else if( !empty($match_html[0]) ){
							if( isset($args['post-format-thumbnail']) && $args['post-format-thumbnail'] === false ){
								$thumbnail_size = 'full';
							}else{
								$thumbnail_size = $args['thumbnail-size'];
							}

							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-video" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
							$ret .= gdlr_core_content_filter($match_html[0]);
							$ret .= '</div>';

							$pages[0] = str_replace($match_html[0], '', $pages[0]);
						}
					}else if( $args['post-format'] == 'audio' ){

						if( !preg_match('#^https?://\S+#', $pages[0], $match) ){
							preg_match('#^\[audio\s.+\[/audio\]#', $pages[0], $match);
						}

						if( !empty($match[0]) ){
							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-audio gdlr-core-sync-height-space" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
							$ret .= gdlr_core_get_audio($match[0]);
							$ret .= '</div>';

							$pages[0] = substr($pages[0], strlen($match[0]));
						}

					}else if( $args['post-format'] == 'image' ){
						if( preg_match('#^<a.+<img.+/></a>|^<img.+/>#', $pages[0], $match) ){ 
							$post_format_image = $match[0];
						}else if( preg_match('#^https?://\S+#', $pages[0], $match) ){
							$post_format_image = gdlr_core_get_image($match[0]);
						}else if( preg_match('#<figure.+</figure>#', $pages[0], $match) ){
							$str_replace = true;
							$post_format_image = $match[0];
						}

						if( !empty($post_format_image) ){
							$thumbnail_wrap_class  = '';
							if( empty($args['opacity-on-hover']) || $args['opacity-on-hover'] == 'enable' ){
								$thumbnail_wrap_class .= ' gdlr-core-opacity-on-hover';
							}
							if( empty($args['zoom-on-hover']) || $args['zoom-on-hover'] == 'enable' ){
								$thumbnail_wrap_class .= ' gdlr-core-zoom-on-hover';
							}else if( $args['zoom-on-hover'] == 'zoom-rotate' ){
								$thumbnail_wrap_class .= ' gdlr-core-zoom-rotate-on-hover';
							}
							if( !empty($args['grayscale-effect']) && $args['grayscale-effect'] == 'enable' ){
								$thumbnail_wrap_class .= ' gdlr-core-grayscale-effect';
							}
							
							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-image ' . esc_attr($thumbnail_wrap_class) . '" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
							$ret .= $post_format_image;
							$ret .= '</div>';

							if( !empty($str_replace) ){
								$pages[0] = str_replace($post_format_image, '', $pages[0]);
							}else{
								$pages[0] = substr($pages[0], strlen($match[0]));
							}
							
						}

					}else if( $args['post-format'] == 'gallery' ){
						if( preg_match('#\[gallery[^\]]+]#', $pages[0], $match) ){ 
							$pages[0] = substr($pages[0], strlen($match[0]));
							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-gallery" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';

							// convert the gallery to slider
							if( !empty($args['post-format-gallery']) && $args['post-format-gallery'] == 'slider' ){
								if( preg_match('#^\[gallery.+ids\s?=\s?\"([^\"]+).+]#', $match[0], $match2) ){
									
									$gallery_atts = array(
										'gallery'=>array(),
										'thumbnail-size'=>$args['thumbnail-size'],
										'style'=>'slider',
										'padding-bottom' => '0px',
										'no-pdlr' => true
									);
									$gallery_ids = explode(',', $match2[1]);
									foreach( $gallery_ids as $gallery_id ){
										$gallery_atts['gallery'][] = array( 'id' => $gallery_id );
									}
									$ret .= gdlr_core_pb_element_gallery::get_content($gallery_atts);

								}

							// display gallery as it is
							}else{
								$ret .= do_shortcode($match[0]);
							}
							$ret .= '</div>';
						}else if( true || strpos('wp:gallery', $pages[0]) !== false ){
							$start = strpos($pages[0], '<!-- wp:gallery');
							$end = strpos($pages[0], '<!-- /wp:gallery -->');

							$gallery = substr($pages[0], $start, $end - $start);

							$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-gallery" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
							$ret .= $gallery;
							$ret .= '</div>';
						}
					}

				}else{

					$feature_image = get_post_thumbnail_id();

					if( !empty($feature_image) ){
						$thumbnail_wrap_class  = '';
						if( empty($args['opacity-on-hover']) || $args['opacity-on-hover'] == 'enable' ){
							$thumbnail_wrap_class .= ' gdlr-core-opacity-on-hover';
						}
						if( empty($args['zoom-on-hover']) || $args['zoom-on-hover'] == 'enable' ){
							$thumbnail_wrap_class .= ' gdlr-core-zoom-on-hover';
						}else if( $args['zoom-on-hover'] == 'zoom-rotate' ){
							$thumbnail_wrap_class .= ' gdlr-core-zoom-rotate-on-hover';
						}
						if( !empty($args['grayscale-effect']) && $args['grayscale-effect'] == 'enable' ){
							$thumbnail_wrap_class .= ' gdlr-core-grayscale-effect';
						}
							
						$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-image ' . esc_attr($thumbnail_wrap_class) . '" ' . gdlr_core_esc_style($thumbnail_atts) . ' >';
						$ret .= '<a href="' . get_permalink() . '" >';
						$ret .= gdlr_core_get_image($feature_image, $args['thumbnail-size'], array('placeholder' => false));
						if( is_sticky() ){
							$ret .= '<div class="gdlr-core-sticky-banner gdlr-core-title-font" ><i class="fa fa-bolt" ></i>' . esc_html__('Sticky Post', 'goodlayers-core') . '</div>';
						}
						$ret .= '</a>';

						if( !empty($args['thumbnail-content']) ){
							$ret .= $args['thumbnail-content'];
						}
						$ret .= '</div>';
					}else{
						if( is_sticky() ){
							$ret .= '<div class="gdlr-core-sticky-banner gdlr-core-title-font" ><i class="fa fa-bolt" ></i>' . esc_html__('Sticky Post', 'goodlayers-core') . '</div>';
						}
					}
				}
				
				return $ret;
			}
			
			// get the blog date
			function blog_date( $args, $order = array('d', 'M') ){

				if( !empty($args['blog-date-feature']) && $args['blog-date-feature'] == 'disable' ) return;
				
				$order = apply_filters('gdlr_core_blog_date_order', $order);

				$ret  = '<div class="gdlr-core-blog-date-wrapper gdlr-core-skin-divider">';
				foreach( $order as $date ){
					switch( $date ){
						case 'd':
							$ret .= '<div class="gdlr-core-blog-date-day gdlr-core-skin-caption">' .  get_the_time('d') . '</div>';
							break;
						case 'M': 
							$ret .= '<div class="gdlr-core-blog-date-month gdlr-core-skin-caption">' . get_the_time('M') . '</div>';
							break;
						case 'Y': 
							$ret .= '<div class="gdlr-core-blog-date-year gdlr-core-skin-caption">' .  get_the_time('Y') . '</div>';
							break;
					}
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
			// get the blog info
			function blog_info( $args ){
				
				$ret = '';
				
				if( !empty($args['display']) ){
					foreach( $args['display'] as $blog_info ){
						
						$ret_temp = '';
						
						switch( $blog_info ){
							case 'date':

								if( get_post_type() == 'post' ){
									$ret_temp .= '<a href="' . get_day_link(get_the_time('Y'), get_the_time('m'), get_the_time('d')) . '" ' . gdlr_core_esc_style(array(
										'color' => empty($args['thumbnail-date-color'])? '': $args['thumbnail-date-color']
									)) . '>';
									$ret_temp .= get_the_date();
									$ret_temp .= '</a>';
								}else{
									$ret_temp .= get_the_date();
								}
								
								break;
								
							case 'tag':
								$ret_temp .= get_the_term_list(get_the_ID(), 'post_tag', '', '<span class="gdlr-core-sep">,</span> ' , '');							
								break;
								
							case 'category':
								$ret_temp .= get_the_term_list(get_the_ID(), 'category', '', '<span class="gdlr-core-sep">,</span> ' , '' );					
								break;
								
							case 'comment-number':
								if( (!isset($args['icon']) || $args['icon'] !== false) && !empty($this->blog_info_prefix[$blog_info]) ){
									$ret_temp .= '<a href="' . get_permalink() . '#respond" >';
									$ret_temp .= get_comments_number() . ' ';
									$ret_temp .= '</a>';
									break;
								}
							
							case 'comment':
								ob_start();
								comments_number(
									esc_html__('No comments', 'goodlayers-core'), 
									esc_html__('1 comment', 'goodlayers-core'), 
									esc_html__('% comments', 'goodlayers-core') 
								);
								$ret_temp .= '<a href="' . get_permalink() . '#respond" >';
								$ret_temp .= ob_get_contents();
								$ret_temp .= '</a>';
								ob_end_clean();								
								break;
								
							case 'author':

								ob_start();
								the_author_posts_link();
								$ret_temp .= ob_get_contents();
								ob_end_clean();					
								break;
						} // switch
						
						if( !empty($ret_temp) ){
							
							$ret .= '<span class="gdlr-core-blog-info gdlr-core-blog-info-font gdlr-core-skin-caption gdlr-core-blog-info-' . esc_attr($blog_info) . '" ';
							if( $blog_info == 'category' ){
								$ret .= gdlr_core_esc_style(array(
									'background' => empty($args['category-background-color'])? '': $args['category-background-color']
								));
							}else{
								$ret .= gdlr_core_esc_style(array(
									'color' => empty($args['thumbnail-date-color'])? '': $args['thumbnail-date-color'],
									'background' => empty($args['thumbnail-date-background-color'])? '': $args['thumbnail-date-background-color']
								));
							}
							$ret .= ' >';
							if( !empty($args['separator']) ){
								$args['separator'] = apply_filters('gdlr_core_blog_info_sep', $args['separator']);
								$ret .= '<span class="gdlr-core-blog-info-sep" >' . $args['separator'] . '</span>';
							}
							if( (!isset($args['icon']) || $args['icon'] !== false) && !empty($this->blog_info_prefix[$blog_info]) ){
								$ret .= '<span class="gdlr-core-head" >' . $this->blog_info_prefix[$blog_info] . '</span>';
							}
							$ret .= $ret_temp;
							$ret .= '</span>';
						}
						
					} // foreach
				} // $args['display']
				
				if( !empty($ret) && !empty($args['wrapper']) ){
					$ret = '<div class="gdlr-core-blog-info-wrapper gdlr-core-skin-divider" >' . $ret . '</div>';
				}
				
				return $ret;
			}

			// blog title
			function blog_title( $args, $permalink = '' ){

				$ret  = '<h3 class="gdlr-core-blog-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($args['blog-title-font-size'])? '': $args['blog-title-font-size'],
					'font-style' => empty($args['blog-title-font-style'])? '': $args['blog-title-font-style'],
					'font-weight' => empty($args['blog-title-font-weight'])? '': $args['blog-title-font-weight'],
					'letter-spacing' => empty($args['blog-title-letter-spacing'])? '': $args['blog-title-letter-spacing'],
					'text-transform' => (empty($args['blog-title-text-transform']) || $args['blog-title-text-transform'] == 'none')? '': $args['blog-title-text-transform']
				)) . ' >';
				if( empty($permalink) ){
					$ret .= '<a href="' . get_permalink() . '" >';
				}else{
					$ret .= '<a href="' . esc_attr($permalink) . '" target="_blank" >';
				}
				$ret .= get_the_title();
				$ret .= '</a>';
				$ret .= '</h3>';

				return  $ret;
			}

			// blog aside
			function blog_format( $args, $post_format ){

				$extra_class  = empty($args['extra-class'])? '': $args['extra-class'];
				$extra_class .= (strpos($args['blog-style'], 'blog-column') === false)? ' gdlr-core-item-list': '';
				$extra_class .= in_array($args['blog-style'], array('blog-column-with-frame', 'blog-full-with-frame'))? ' gdlr-core-with-frame gdlr-core-item-mgb': '';
				$extra_class .= !empty($args['sync-height'])? ' gdlr-core-js': '';

				$frame_shadow = array();
				if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$frame_shadow['background-shadow-size'] = $args['frame-shadow-size'];
					$frame_shadow['background-shadow-color'] = $args['frame-shadow-color'];
					$frame_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
				}

				$ret  = '<div class="gdlr-core-blog-' . esc_attr($post_format) . '-format ' . esc_attr($extra_class) . '" ';
				$ret .= !empty($args['sync-height'])? ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" data-sync-height-center':'';
				$ret .= ' ' . gdlr_core_esc_style($frame_shadow) . '>';
				if( $post_format == 'aside' ){

					if( empty($args['excerpt-number']) ){
						$args['excerpt-number'] = 35;
					}
					$ret .= $this->get_blog_excerpt($args);

				}else if( $post_format == 'quote' ){

					global $pages;

					if( !preg_match('#^\s*\[gdlr_core_quote[\s\S]+\[/gdlr_core_quote\]#', $pages[0], $match) ){ 
						preg_match('#^\s*<blockquote[\s\S]+</blockquote>#', $pages[0], $match);
					}

					if( !empty($match[0]) ){
						$blockquote = $match[0];
						$author = substr($pages[0], strlen($match[0]));
					}else{
						$blockquote = '';
						$author = $pages[0];
					}

					$ret .= '<div class="gdlr-core-blog-content" >';
					$thumbnail_id = get_post_thumbnail_id();
					if( !empty($thumbnail_id) ){
						$quote_background = gdlr_core_get_image_url(get_post_thumbnail_id());
						$ret .= '<div class="gdlr-core-blog-quote-background" ' . gdlr_core_esc_style(array(
							'background-image' => $quote_background
						)) . ' ></div>';
					}

					$ret .= '<div class="gdlr-core-blog-quote gdlr-core-quote-font" >&#8220;</div>';
					
					$ret .= '<div class="gdlr-core-blog-content-wrap" >';
					if( !empty($blockquote) ){
						$ret .= '<div class="gdlr-core-blog-quote-content gdlr-core-info-font">' . gdlr_core_content_filter($blockquote, true) . '</div>';
					}
					if( !empty($author) ){
						$ret .= '<div class="gdlr-core-blog-quote-author gdlr-core-info-font" >' . gdlr_core_text_filter($author) . '</div>';
					}
					$ret .= '</div>';
					$ret .= '</div>';

				}else if( $post_format == 'link' ){

					global $pages;

					if( preg_match('#^<a.+href=[\'"]([^\'"]+).+</a>#', $pages[0], $match) ){ 
						$post_format_link = $match[1];
						$pages[0] = substr($pages[0], strlen($match[0]));
					}else if( preg_match('#(https?://\S+)<#', $pages[0], $match) ){
						$post_format_link = $match[1];
						$index = strpos($pages[0], $match[1]);
						$pages[0] = substr_replace($pages[0], '', $index, strlen($match[1]));
					}else{
						$post_format_link = get_permalink();
					}

					$ret .= '<div class="gdlr-core-blog-content-outer-wrap" >';
					$ret .= '<a class="gdlr-core-blog-icon-link" href="' . esc_url($post_format_link) . '" target="_blank" ><i class="icon_link" ></i></a>';

					$ret .= '<div class="gdlr-core-blog-content-wrap" >';
					$ret .= $this->blog_title( $args, $post_format_link );
					$ret .= $this->get_blog_excerpt( $args );
					$ret .= '</div>';
					$ret .= '</div>';

				}
				$ret .= '</div>';

				return $ret;
			}
			
			// blog full
			function blog_full( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-full gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';

					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';
				$additional_class .= (!empty($args['blog-full-alignment']))? ' gdlr-core-style-' . $args['blog-full-alignment']: '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-full-with-frame' ){
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-full ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-thumbnail' => false,
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					), $thumbnail_shadow); 
				}
				
				if( $args['blog-style'] == 'blog-full-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-full-frame gdlr-core-skin-e-background" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}
				$ret .= '<div class="gdlr-core-blog-full-head clearfix">';
				$ret .= $this->blog_date($args);
				
				$ret .= '<div class="gdlr-core-blog-full-head-right">';
				$ret .= $this->blog_title( $args );
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true
				));
				$ret .= '</div>'; // gdlr-core-blog-full-head-right
				$ret .= '</div>'; // gdlr-core-blog-full-head
				
				$ret .= $this->get_blog_excerpt($args);
				
				$ret .= ($args['blog-style'] == 'blog-full-with-frame')? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-full
				
				return $ret;
			}
			function blog_full_style_2( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-full gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';

					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';
				$additional_class .= (!empty($args['blog-full-alignment']))? ' gdlr-core-style-' . $args['blog-full-alignment']: '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-full-with-frame' ){
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-full gdlr-core-style-2 ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$thumb_cat = array();
					$thumb_content = '';
					if( in_array('category', $args['meta-option']) ){
						$thumb_cat[] = 'category';
					}
					if( in_array('comment-number', $args['meta-option']) ){
						$thumb_cat[] = 'comment-number';
					}
					if( !empty($thumb_cat) ){
						$thumb_content = '<span class="gdlr-core-blog-thumbnail-content" >' . $this->blog_info(array(
							'display' => $thumb_cat,
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						)) . '</span>';
					}

					$blog_thumbnail = $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-thumbnail' => false,
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
						'thumbnail-content' => $thumb_content
					), $thumbnail_shadow); 

					$ret .= $blog_thumbnail;
					if( !empty($blog_thumbnail) && empty($post_format) ){ 
						$args['meta-option'] = array_diff($args['meta-option'], array('comment-number', 'category'));
					}
				}
				
				if( $args['blog-style'] == 'blog-full-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-full-frame gdlr-core-skin-e-background" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}
				$ret .= '<div class="gdlr-core-blog-full-head clearfix">';
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));

				$ret .= $this->blog_title( $args );
				$ret .= '</div>'; // gdlr-core-blog-full-head
				
				$ret .= $this->get_blog_excerpt($args);
				
				$ret .= ($args['blog-style'] == 'blog-full-with-frame')? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-full
				
				return $ret;
			}
			function blog_full_style_2_date( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-full gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';

					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';
				$additional_class .= (!empty($args['blog-full-alignment']))? ' gdlr-core-style-' . $args['blog-full-alignment']: '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-full-with-frame' ){
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-full gdlr-core-style-2-date ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-thumbnail' => false,
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
					), $thumbnail_shadow); 
				}
				
				if( $args['blog-style'] == 'blog-full-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-full-frame gdlr-core-skin-e-background" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}

				if( in_array('date', $args['meta-option']) ){ 	
					$ret .= '<div class="gdlr-core-blog-full-date" >' . $this->blog_info(array(
						'display' => array('date'),
						'wrapper' => false,
						'thumbnail-date-background-color' => empty($args['thumbnail-date-background-color'])? '': $args['thumbnail-date-background-color'],
						'thumbnail-date-color' => empty($args['thumbnail-date-color'])? '': $args['thumbnail-date-color'],
					)) . '</div>';
					$args['meta-option'] = array_diff($args['meta-option'], array('date'));
				}

				$ret .= '<div class="gdlr-core-blog-full-head clearfix">';
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));

				$ret .= $this->blog_title( $args );
				$ret .= '</div>'; // gdlr-core-blog-full-head
				
				$ret .= $this->get_blog_excerpt($args);
				
				$ret .= ($args['blog-style'] == 'blog-full-with-frame')? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-full
				
				return $ret;
			}
			function blog_full_style_2_tag( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-full gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';

					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';
				$additional_class .= (!empty($args['blog-full-alignment']))? ' gdlr-core-style-' . $args['blog-full-alignment']: '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-full-with-frame' ){
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-full gdlr-core-style-2-tag ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$blog_thumbnail = $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-thumbnail' => false,
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					), $thumbnail_shadow); 

					$ret .= $blog_thumbnail;
				}
				
				$display_tag = false;
				if( in_array('tag', $args['meta-option']) ){
					$display_tag = true;
					$args['meta-option'] = array_diff($args['meta-option'], array('tag'));
				}

				if( $args['blog-style'] == 'blog-full-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-full-frame gdlr-core-skin-e-background" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}
				$ret .= '<div class="gdlr-core-blog-full-head clearfix">';
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));

				$ret .= $this->blog_title( $args );
				$ret .= '</div>'; // gdlr-core-blog-full-head
				
				$ret .= $this->get_blog_excerpt($args);

				if( $display_tag ){
					$ret .= $this->blog_info(array(
						'display' => array('tag'),
						'wrapper' => false
					));
				}
				
				$ret .= ($args['blog-style'] == 'blog-full-with-frame')? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-full
				
				return $ret;
			}
			function blog_full_style_3( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-full gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';

					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-mglr';
				$additional_class .= (!empty($args['blog-full-alignment']))? ' gdlr-core-style-' . $args['blog-full-alignment']: '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-full-with-frame' ){
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$thumbnail_atts = array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-thumbnail' => false,
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					);
					if( $args['blog-style'] != 'blog-full-with-frame' && in_array('category', $args['meta-option']) ){
						$thumbnail_atts['thumbnail-content'] = $this->blog_info(array(
							'display' => array('category'),
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						));
						$args['meta-option'] = array_diff($args['meta-option'], array('category'));
					}
					$blog_thumbnail = $this->blog_thumbnail($thumbnail_atts, $thumbnail_shadow);

					if( !empty($blog_thumbnail) ){
						$additional_class .= ' gdlr-core-with-thumbnail';
					}
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-full gdlr-core-style-3 ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				if( !empty($blog_thumbnail) ){
					$ret .= $blog_thumbnail;
				}
				if( $args['blog-style'] == 'blog-full-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-full-frame gdlr-core-skin-e-background" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}
				
				if( $args['blog-style'] == 'blog-full-with-frame' && in_array('category', $args['meta-option']) ){
					$ret .= $this->blog_info(array(
						'display' => array('category'),
						'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
					));
					$args['meta-option'] = array_diff($args['meta-option'], array('category'));
				}

				$ret .= '<div class="gdlr-core-blog-full-head clearfix">';
				$ret .= $this->blog_title( $args );
				$ret .= '</div>'; // gdlr-core-blog-full-head
				
				$read_more_button = $this->blog_excerpt_read_more_button($args);
				$args['show-read-more'] = 'none';
				$ret .= $this->get_blog_excerpt($args);

				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= $read_more_button;
				$ret .= ($args['blog-style'] == 'blog-full-with-frame')? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-full
				
				return $ret;
			}
			
			// blog medium
			function blog_medium( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-medium gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-pdlr';
					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = empty($args['blog-style'])? '': 'gdlr-core-' . $args['blog-style'];
				$additional_class .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-pdlr';
				if( !empty($args['blog-side-thumbnail-style']) && $args['blog-side-thumbnail-style'] == 'style-1-large' ){
					$additional_class .= ' gdlr-core-large';
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-medium clearfix ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style(array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				)) . ' >';

				// left thumbnail
				if( $args['blog-style'] == 'blog-left-thumbnail' ){
					$ret .= '<div class="gdlr-core-blog-thumbnail-wrap clearfix" >';
					if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
						$thumbnail_shadow = array();
						if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
							$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
							$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
							$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						}
						$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

						$ret .= $this->blog_thumbnail(array(
							'thumbnail-size' => $args['thumbnail-size'],
							'post-format' => ($post_format == 'audio')? '': $post_format,
							'post-format-gallery' => 'slider',
							'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
							'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
							'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
						), $thumbnail_shadow);
					}
					$ret .= $this->blog_date($args);
					$ret .= '</div>';					
				}

				// content
				$ret .= '<div class="gdlr-core-blog-medium-content-wrapper clearfix">';
				if( $post_format == 'audio' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => 'audio',
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					));
				}
				$ret .= $this->blog_title( $args );
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true
				));
				$ret .= $this->get_blog_excerpt($args);
				$ret .= '</div>'; // gdlr-core-blog-medium-content-wrapper

				// right thumbnail
				if( $args['blog-style'] == 'blog-right-thumbnail' ){
					$ret .= '<div class="gdlr-core-blog-thumbnail-wrap clearfix" >';
					if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
						$thumbnail_shadow = array();
						if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
							$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
							$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
							$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						}
						$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

						$ret .= $this->blog_thumbnail(array(
							'thumbnail-size' => $args['thumbnail-size'],
							'post-format' => ($post_format == 'audio')? '': $post_format,
							'post-format-gallery' => 'slider',
							'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
							'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
							'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
						), $thumbnail_shadow);
					}
					$ret .= $this->blog_date($args);
					$ret .= '</div>';					
				}
				$ret .= '</div>'; // gdlr-core-blog-medium
				
				return $ret;
			} 

			// blog medium
			function blog_medium_style_2( $args ){
				
				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-medium gdlr-core-large';
					$args['extra-class'] .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-pdlr';
					return $this->blog_format( $args, $post_format );
				}

				$additional_class  = empty($args['blog-style'])? '': 'gdlr-core-' . $args['blog-style'];
				$additional_class .= (!empty($args['layout']) && $args['layout'] == 'carousel')? '': ' gdlr-core-item-pdlr';
				if( !empty($args['blog-side-thumbnail-style']) || $args['blog-side-thumbnail-style'] == 'style-2-large' ){
					$additional_class .= ' gdlr-core-large';
				}

				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-medium gdlr-core-style-2 clearfix ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style(array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				)) . ' >';

				// left thumbnail
				if( $args['blog-style'] == 'blog-left-thumbnail' ){
					$ret .= '<div class="gdlr-core-blog-thumbnail-wrap clearfix" >';
					if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
						$thumbnail_shadow = array();
						if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
							$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
							$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
							$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						}
						$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

						$thumb_cat = array();
						$thumb_content = '';
						if( in_array('category', $args['meta-option']) ){
							$thumb_cat[] = 'category';
						}
						if( in_array('comment-number', $args['meta-option']) ){
							$thumb_cat[] = 'comment-number';
						}
						if( !empty($thumb_cat) ){
							$thumb_content = $this->blog_info(array(
								'display' => $thumb_cat,
								'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
							));
						}

						$blog_thumbnail = $this->blog_thumbnail(array(
							'thumbnail-size' => $args['thumbnail-size'],
							'post-format' => ($post_format == 'audio')? '': $post_format,
							'post-format-gallery' => 'slider',
							'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
							'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
							'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
							'thumbnail-content' => $thumb_content
						), $thumbnail_shadow);

						$ret .= $blog_thumbnail;
						if( !empty($blog_thumbnail) && empty($post_format) ){ 
							$args['meta-option'] = array_diff($args['meta-option'], array('comment-number', 'category'));
						}
						
					}
					$ret .= '</div>';
				}

				// content
				$ret .= '<div class="gdlr-core-blog-medium-content-wrapper clearfix">';
				if( $post_format == 'audio' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => 'audio',
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					));
				}
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= $this->blog_title( $args );
				$ret .= $this->get_blog_excerpt($args);
				
				$ret .= '</div>'; // gdlr-core-blog-medium-content-wrapper

				// right thumbnail
				if( $args['blog-style'] == 'blog-right-thumbnail' ){
					$ret .= '<div class="gdlr-core-blog-thumbnail-wrap clearfix" >';
					if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
						$thumbnail_shadow = array();
						if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
							$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
							$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
							$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						}
						$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

						$thumb_cat = array();
						$thumb_content = '';
						if( in_array('category', $args['meta-option']) ){
							$thumb_cat[] = 'category';
						}
						if( in_array('comment-number', $args['meta-option']) ){
							$thumb_cat[] = 'comment-number';
						}
						if( !empty($thumb_cat) ){
							$thumb_content = $this->blog_info(array(
								'display' => $thumb_cat,
								'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
							));
						}

						$blog_thumbnail = $this->blog_thumbnail(array(
							'thumbnail-size' => $args['thumbnail-size'],
							'post-format' => ($post_format == 'audio')? '': $post_format,
							'post-format-gallery' => 'slider',
							'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
							'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
							'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
							'thumbnail-content' => $thumb_content
						), $thumbnail_shadow);

						$ret .= $blog_thumbnail;
						if( !empty($blog_thumbnail) && empty($post_format) ){ 
							$args['meta-option'] = array_diff($args['meta-option'], array('comment-number', 'category'));
						}
						
					}
					$ret .= '</div>';
				}

				$ret .= '</div>'; // gdlr-core-blog-medium
				
				return $ret;
			} 			
			
			// blog column hover background
			function blog_column_hover_bg( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-grid gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$ret  = '<div class="gdlr-core-blog-column-hover-background gdlr-core-item-mgb clearfix" >';
				$ret .= '<div class="gdlr-core-background" ';
				$feature_image = get_post_thumbnail_id();
				if( !empty($feature_image) ){
					$ret .= gdlr_core_esc_style(array(
						'background-image' => $feature_image
					));
				}
				$ret .= ' ><div class="gdlr-core-background-overlay" ' . gdlr_core_esc_style(array(
					'background-color' => empty($args['blog-column-thumbnail-overlay-color'])? '': $args['blog-column-thumbnail-overlay-color'],
					'opacity' => empty($args['blog-column-thumbnail-overlay-opacity'])? '': $args['blog-column-thumbnail-overlay-opacity']
				)) . ' ></div></div>';
				$ret .= '<div class="gdlr-core-content" >';
				if( ($key = array_search('date', $args['meta-option'])) !== false ) {
					$ret .= '<div class="gdlr-core-column-date" >';
					$ret .= $this->blog_info(array('display' => array('date')));
					$ret .= '</div>';

					unset($args['meta-option'][$key]);
				}

				$ret .= $this->blog_title($args);

				if( !empty($args['meta-option']) ){
					$ret .= '<div class="gdlr-core-blog-info-wrapper gdlr-core-skin-divider" ';
					if( !empty($args['layout']) && $args['layout'] != 'masonry' ){
						$ret .= 'data-sync-height-offset';
					}
					$ret .= ' >';
					$ret .= $this->blog_info(array(
						'display' => $args['meta-option'],
						'wrapper' => false
					));
					$ret .= '</div>';
				}
				$ret .= '</div>';
				$ret .= '</div>';

				return $ret;
			}

			// blog column
			function blog_grid( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class']  = ' gdlr-core-blog-grid gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';

					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					), $thumbnail_shadow);
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}
				if( ($key = array_search('date', $args['meta-option'])) !== false ) {
					$ret .= '<div class="gdlr-core-blog-grid-date" >';
					$ret .= $this->blog_info(array('display' => array('date')));
					$ret .= '</div>';

					unset($args['meta-option'][$key]);
				}
				
				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				
				if( !empty($args['meta-option']) ){
					$ret .= '<div class="gdlr-core-blog-info-wrapper gdlr-core-skin-divider" ';
					if( !empty($args['layout']) && $args['layout'] != 'masonry' ){
						$ret .= 'data-sync-height-offset';
					}
					$ret .= ' >';
					$ret .= $this->blog_info(array(
						'display' => $args['meta-option'],
						'wrapper' => false
					));
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 		

			// blog column
			function blog_grid_style_2( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-2 gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2 gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2 ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){

					$thumb_cat = array();
					$thumb_content = '';
					if( in_array('category', $args['meta-option']) ){
						$thumb_cat[] = 'category';
					}
					if( in_array('comment-number', $args['meta-option']) ){
						$thumb_cat[] = 'comment-number';
					}
					if( !empty($thumb_cat) ){
						$thumb_content = $this->blog_info(array(
							'display' => $thumb_cat,
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						));
					}

					$blog_thumbnail = $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
						'thumbnail-content' => $thumb_content
					), $thumbnail_shadow);

					$ret .= $blog_thumbnail;
					if( !empty($blog_thumbnail) && empty($post_format) ){ 	
						$args['meta-option'] = array_diff($args['meta-option'], array('comment-number', 'category'));
					}
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 

			// blog column
			function blog_grid_style_2_date( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-2 gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2-date gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2-date ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
					), $thumbnail_shadow);
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}

				if( !in_array($post_format, array('video', 'audio')) ){
					if( in_array('date', $args['meta-option']) ){ 	
						$ret .= '<div class="gdlr-core-blog-grid-date" >' . $this->blog_info(array(
							'display' => array('date'),
							'wrapper' => false,
							'thumbnail-date-background-color' => empty($args['thumbnail-date-background-color'])? '': $args['thumbnail-date-background-color'],
							'thumbnail-date-color' => empty($args['thumbnail-date-color'])? '': $args['thumbnail-date-color'],
						)) . '</div>';
						$args['meta-option'] = array_diff($args['meta-option'], array('date'));
					}
				}

				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 		

			function blog_grid_style_2_tag( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-2-tag gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2-tag gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-2-tag ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){

					$thumb_cat = array();
					$thumb_content = '';
					if( in_array('category', $args['meta-option']) ){
						$thumb_cat[] = 'category';
					}
					if( in_array('comment-number', $args['meta-option']) ){
						$thumb_cat[] = 'comment-number';
					}
					if( !empty($thumb_cat) ){
						$thumb_content = $this->blog_info(array(
							'display' => $thumb_cat,
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						));
					}

					$blog_thumbnail = $this->blog_thumbnail(array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect'],
						'thumbnail-content' => $thumb_content
					), $thumbnail_shadow);

					$ret .= $blog_thumbnail;
					if( !empty($blog_thumbnail) && empty($post_format) ){ 	
						$args['meta-option'] = array_diff($args['meta-option'], array('comment-number', 'category'));
					}
				}

				$display_tag = false;
				if( in_array('tag', $args['meta-option']) ){
					$display_tag = true;
					$args['meta-option'] = array_diff($args['meta-option'], array('tag'));
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);

				if( $display_tag ){
					$ret .= $this->blog_info(array(
						'display' => array('tag'),
						'wrapper' => false
					));
				}
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 

			// blog column
			function blog_grid_style_3( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-3 gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$thumbnail_atts = array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					);
					if( $args['blog-style'] != 'blog-column-with-frame' && in_array('category', $args['meta-option']) ){
						$thumbnail_atts['thumbnail-content'] = $this->blog_info(array(
							'display' => array('category'),
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						));
						$args['meta-option'] = array_diff($args['meta-option'], array('category'));
					}
					$blog_thumbnail = $this->blog_thumbnail($thumbnail_atts, $thumbnail_shadow);

					if( !empty($blog_thumbnail) ){
						$additional_class .= ' gdlr-core-with-thumbnail';
					} 
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-3 gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-3 ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( !empty($blog_thumbnail) ){
					$ret .= $blog_thumbnail;
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}
				
				if($args['blog-style'] == 'blog-column-with-frame' && in_array('category', $args['meta-option']) ){
					$ret .= $this->blog_info(array(
						'display' => array('category'),
						'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
					));
					$args['meta-option'] = array_diff($args['meta-option'], array('category'));
				}

				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => '•'
				));
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 	

			// blog column
			function blog_grid_style_3_date( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-3 gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = '';

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
					
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$thumbnail_atts = array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					);
					$blog_thumbnail = $this->blog_thumbnail($thumbnail_atts, $thumbnail_shadow);

					if( !empty($blog_thumbnail) ){
						$additional_class .= ' gdlr-core-with-thumbnail';
					} 
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-3-date gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-3-date ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( !empty($blog_thumbnail) ){
					if( in_array('date', $args['meta-option']) ){
						$ret .= $this->blog_info(array(
							'display' => array('date'),
							'thumbnail-date-background-color' => empty($args['thumbnail-date-background-color'])? '': $args['thumbnail-date-background-color'],
							'thumbnail-date-color' => empty($args['thumbnail-date-color'])? '': $args['thumbnail-date-color'],
						));
						$args['meta-option'] = array_diff($args['meta-option'], array('date'));
					}
					$ret .= $blog_thumbnail;
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap">';
				}

				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
				));
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 	

			function blog_grid_style_4( $args ){

				$post_format = get_post_format();
				if( in_array($post_format, array('aside', 'quote', 'link')) ){
					$args['extra-class'] = ' gdlr-core-blog-grid gdlr-core-style-3 gdlr-core-small';
					if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
						$args['sync-height'] = true;
					}
					return $this->blog_format( $args, $post_format );
				}

				$additional_class = empty($args['additional-class'])? '': $args['additional-class'];

				// shadow
				$blog_atts = array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				);
				$thumbnail_shadow = array();
				if($args['blog-style'] == 'blog-column-with-frame' ){
					$additional_class .= ' gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background ';
					
					$blog_atts['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$blog_atts['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					$blog_atts['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$blog_atts['background-shadow-size'] = $args['frame-shadow-size'];
						$blog_atts['background-shadow-color'] = $args['frame-shadow-color'];
						$blog_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];
						$additional_class .= ' gdlr-core-outer-frame-element';
					}
				}else{
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}
					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
				}

				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$thumbnail_atts = array(
						'thumbnail-size' => $args['thumbnail-size'],
						'post-format' => $post_format,
						'post-format-gallery' => 'slider',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					);
					$blog_thumbnail = $this->blog_thumbnail($thumbnail_atts, $thumbnail_shadow);

					if( !empty($blog_thumbnail) ){
						$additional_class .= ' gdlr-core-with-thumbnail';
					} 
				}

				// move up with shadow effect
				if( $args['blog-style'] == 'blog-column-with-frame' && !empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element';
				}

				if($args['blog-style'] == 'blog-column-with-frame' && $args['layout'] != 'masonry' ){
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-4 gdlr-core-js ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts);
					$ret .= ' data-sync-height="blog-item-' . esc_attr($this->blog_item_id) . '" ';
					if( $post_format == 'audio' ){
						$ret .= ' data-sync-height-center';
					}
					$ret .= ' >';
				}else{
					$ret  = '<div class="gdlr-core-blog-grid gdlr-core-style-4 ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($blog_atts) . ' >';
				}

				if( !empty($blog_thumbnail) ){
					$ret .= $blog_thumbnail;
				}
				
				if( $args['blog-style'] == 'blog-column-with-frame' ){
					$ret .= '<div class="gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position clearfix" ' . gdlr_core_esc_style(array(
						'padding' => empty($args['blog-frame-padding'])? '': $args['blog-frame-padding'],
					)) . ' >';
				}else{
					$ret .= '<div class="gdlr-core-blog-grid-content-wrap clearfix">';
				}
				
				$meta = '';
				if( in_array('date', $args['meta-option']) ){
					$meta .= $this->blog_info(array('display' => array('date')));
					$args['meta-option'] = array_diff($args['meta-option'], array('date'));
				}
				if( in_array('tag', $args['meta-option']) ){
					$meta .= $this->blog_info(array('display' => array('tag')));
					$args['meta-option'] = array_diff($args['meta-option'], array('tag'));
				}
				if( !empty($meta) ){
					$ret .= '<div class="gdlr-core-blog-grid-top-info clearfix" >' . $meta . '</div>';
				}
				

				$ret .= $this->blog_title($args);
				$ret .= $this->get_blog_excerpt($args);
				$ret .= '</div>'; // gdlr-core-blog-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-blog-grid
				
				return $ret;
			} 		

			// blog modern
			function blog_modern( $args ){
				
				$thumbnail_shadow = array();
				$feature_image = get_post_thumbnail_id();

				if( !empty($args['blog-image-style']) && $args['blog-image-style'] == 'style-5'){
					$additional_class  = ' gdlr-core-no-image gdlr-core-skin-e-background gdlr-core-skin-border';
					
					if( !empty($args['blog-image-5-background']) ){
						$thumbnail_shadow['background-color'] = $args['blog-image-5-background'];
					}
				}else{
					$additional_class  = empty($feature_image)? ' gdlr-core-no-image': ' gdlr-core-with-image';
				}
			
				
				if( empty($args['always-show-overlay-content']) || $args['always-show-overlay-content'] == 'disable' ){
					$additional_class .= ' gdlr-core-hover-overlay-content';
				}
				if( empty($args['enable-thumbnail-opacity-on-hover']) || $args['enable-thumbnail-opacity-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-opacity-on-hover';
				}
				if( empty($args['enable-thumbnail-zoom-on-hover']) || $args['enable-thumbnail-zoom-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-zoom-on-hover';
				}else if( $args['enable-thumbnail-zoom-on-hover'] == 'zoom-rotate' ){
					$additional_class .= ' gdlr-core-zoom-rotate-on-hover';
				}
				if( !empty($args['enable-thumbnail-grayscale-effect']) && $args['enable-thumbnail-grayscale-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-grayscale-effect';
				}
				if( !empty($args['blog-image-thumbnail-overlay']) && $args['blog-image-thumbnail-overlay'] == 'gradient-slide' ){
					$additional_class .= ' gdlr-core-gradient-slide';
				}
				if( !empty($args['blog-image-style']) ){
					$additional_class .= ' gdlr-core-' . $args['blog-image-style'];
				}
				
				if( !empty($feature_image) ){
					$thumbnail_shadow['border-width'] = empty($args['blog-frame-border-size'])? '': $args['blog-frame-border-size'];
					$thumbnail_shadow['border-color'] = empty($args['blog-frame-border-color'])? '': $args['blog-frame-border-color'];
					
					if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
						$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
						$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
						$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
					}

					$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];

					$additional_class .= ' gdlr-core-outer-frame-element';
				}

				$ret  = '<div class="gdlr-core-blog-modern ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($thumbnail_shadow) . ' >';
				$ret .= '<div class="gdlr-core-blog-modern-inner">';

				if( !empty($feature_image) ){
					$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-image" >';
					$ret .= gdlr_core_get_image($feature_image, $args['thumbnail-size'], array(
						'placeholder' => false,
						'img-style' => gdlr_core_esc_style(array(
							'opacity' => empty($args['blog-image-initial-opacity'])? '': $args['blog-image-initial-opacity']
						))
					));
					if( !empty($args['blog-image-style']) && $args['blog-image-style'] == 'style-2' && in_array('category', $args['meta-option']) ){
						$ret .=  $this->blog_info(array(
							'display' => array('category'),
							'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
						));
						$args['meta-option'] = array_diff($args['meta-option'], array('category'));
					}
					$ret .= '</div>';
				}
				
				if( !empty($args['blog-image-thumbnail-overlay']) ){
					if( $args['blog-image-thumbnail-overlay'] == 'gradient' ){
						$ret .= '<div class="gdlr-core-blog-modern-content-overlay-gradient" ></div>';
					}else if( $args['blog-image-thumbnail-overlay'] == 'gradient2' ){
						$ret .= '<div class="gdlr-core-blog-modern-content-overlay-gradient2" ></div>';
					}else if( $args['blog-image-thumbnail-overlay'] == 'opacity' ){
						$ret .= '<div class="gdlr-core-blog-modern-content-overlay" ' . gdlr_core_esc_style(array(
							'background-color' => (empty($args['blog-image-thumbnail-overlay-color']) || $args['blog-image-thumbnail-overlay-color'] == '#000000')? '': $args['blog-image-thumbnail-overlay-color'],
							'opacity' => (empty($args['blog-image-thumbnail-overlay-opacity']) || $args['blog-image-thumbnail-overlay-opacity'] == '0.4')? '': $args['blog-image-thumbnail-overlay-opacity']
						)) . ' ></div>';
					}
				}
				$ret .= '<div class="gdlr-core-blog-modern-content ';
				$ret .= empty($args['blog-image-alignment'])? ' gdlr-core-center-align': ' gdlr-core-' . esc_attr($args['blog-image-alignment']) . '-align';
				$ret .= '" ' . gdlr_core_esc_style(array(
					'padding' => empty($args['blog-image-overlay-content-padding'])? '': $args['blog-image-overlay-content-padding']
				)) . '>';
				$ret .= $this->blog_title($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => empty($args['blog-image-style'])? '': '•'
				));
				if( !empty($args['blog-image-excerpt-number']) ){
					$args['excerpt'] = 'specify-number';
					$args['excerpt-number'] = $args['blog-image-excerpt-number'];
					$ret .= $this->get_blog_excerpt($args);
				}
				$ret .= '</div>'; // gdlr-core-blog-modern-content
				$ret .= '</div>'; // gdlr-core-blog-modern-inner
				$ret .= '</div>'; // gdlr-core-blog-modern
				
				return $ret;
			} 		

			// blog feature
			function blog_feature( $args ){
				
				$feature_image = get_post_thumbnail_id();
				$additional_class  = empty($feature_image)? ' gdlr-core-no-image': ' gdlr-core-with-image';
				if( empty($args['blog-feature-main']) ){
					$additional_class .= ' gdlr-core-sub-item';

					unset($args['blog-title-font-size']);
				}else{
					$additional_class .= ' gdlr-core-main-item';
				}
				if( empty($args['always-show-overlay-content']) || $args['always-show-overlay-content'] == 'disable' ){
					$additional_class .= ' gdlr-core-hover-overlay-content';
				}
				if( empty($args['enable-thumbnail-opacity-on-hover']) || $args['enable-thumbnail-opacity-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-opacity-on-hover';
				}
				if( empty($args['enable-thumbnail-zoom-on-hover']) || $args['enable-thumbnail-zoom-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-zoom-on-hover';
				}else if( $args['enable-thumbnail-zoom-on-hover'] == 'zoom-rotate' ){
					$additional_class .= ' gdlr-core-zoom-rotate-on-hover';
				}
				if( !empty($args['enable-thumbnail-grayscale-effect']) && $args['enable-thumbnail-grayscale-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-grayscale-effect';
				}

				$ret  = '<div class="gdlr-core-blog-feature ' . esc_attr($additional_class) . '" >';
				$ret .= '<div class="gdlr-core-blog-feature-inner">';

				if( !empty($feature_image) ){
					$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-image" >';
					$ret .= gdlr_core_get_image($feature_image, $args['thumbnail-size'], array('placeholder' => false));
					$ret .= '</div>';
				}
				
				if( !empty($args['blog-image-thumbnail-overlay']) ){
					if( $args['blog-image-thumbnail-overlay'] == 'gradient' ){
						$ret .= '<div class="gdlr-core-blog-feature-content-overlay-gradient" ></div>';
					}else if( $args['blog-image-thumbnail-overlay'] == 'gradient2' ){
						$ret .= '<div class="gdlr-core-blog-feature-content-overlay-gradient2" ></div>';
					}else if( $args['blog-image-thumbnail-overlay'] == 'opacity' ){
						$ret .= '<div class="gdlr-core-blog-feature-content-overlay" ' . gdlr_core_esc_style(array(
							'opacity' => (empty($args['blog-image-thumbnail-overlay-opacity']) || $args['blog-image-thumbnail-overlay-opacity'] == '0.4')? '': $args['blog-image-thumbnail-overlay-opacity']
						)) . ' ></div>';
					}
				}

				if( in_array('category', $args['meta-option']) ){
					$ret .= $this->blog_info(array(
						'display' => array('category'),
						'category-background-color' => empty($args['category-background-color'])? '': $args['category-background-color']
					));
					$args['meta-option'] = array_diff($args['meta-option'], array('category'));
				}

				$ret .= '<div class="gdlr-core-blog-feature-content">';
				$ret .= $this->blog_title($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true
				));
				$ret .= '</div>'; // gdlr-core-blog-modern-content
				$ret .= '</div>'; // gdlr-core-blog-modern-inner
				$ret .= '</div>'; // gdlr-core-blog-modern
				
				return $ret;
			} 			

			// blog metro
			function blog_metro( $args ){
				
				$feature_image = get_post_thumbnail_id();
				$additional_class  = empty($feature_image)? ' gdlr-core-no-image': ' gdlr-core-with-image';
				if( empty($args['enable-thumbnail-opacity-on-hover']) || $args['enable-thumbnail-opacity-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-opacity-on-hover';
				}
				if( empty($args['enable-thumbnail-zoom-on-hover']) || $args['enable-thumbnail-zoom-on-hover'] == 'enable' ){
					$additional_class .= ' gdlr-core-zoom-on-hover';
				}else if( $args['enable-thumbnail-zoom-on-hover'] == 'zoom-rotate' ){
					$additional_class .= ' gdlr-core-zoom-rotate-on-hover';
				}
				if( !empty($args['enable-thumbnail-grayscale-effect']) && $args['enable-thumbnail-grayscale-effect'] == 'enable' ){
					$additional_class .= ' gdlr-core-grayscale-effect';
				}

				$thumbnail_shadow = array();
				if( !empty($feature_image) && !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$thumbnail_shadow['background-shadow-size'] = $args['frame-shadow-size'];
					$thumbnail_shadow['background-shadow-color'] = $args['frame-shadow-color'];
					$thumbnail_shadow['background-shadow-opacity'] = $args['frame-shadow-opacity'];
				}
				$thumbnail_shadow['border-radius'] = empty($args['blog-border-radius'])? '': $args['blog-border-radius'];
					
				$ret  = '<div class="gdlr-core-blog-metro ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($thumbnail_shadow) . ' >';
				$ret .= '<div class="gdlr-core-blog-metro-inner ' .  ((!empty($feature_image) && $args['blog-style'] == 'blog-metro')? ' gdlr-core-metro-rvpdlr': '') . '" >';

				if( !empty($feature_image) ){
					$ret .= '<div class="gdlr-core-blog-thumbnail gdlr-core-media-image" >';
					$ret .= gdlr_core_get_image($feature_image, $args['thumbnail-size'], array('placeholder' => false));
					$ret .= '</div>';
				}
				
				$ret .= '<div class="gdlr-core-blog-metro-content">';
				$ret .= $this->blog_title($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true
				));
				$ret .= '</div>'; // gdlr-core-blog-metro-content
				$ret .= '</div>'; // gdlr-core-blog-metro-inner
				$ret .= '</div>'; // gdlr-core-blog-metro
				
				return $ret;
			} 			

			// blog list
			function blog_widget( $args, $featured = false ){

				$additional_class  = empty($args['text-align'])? '': ' gdlr-core-' . $args['text-align'] . '-align';
				$additional_class .= empty($args['item-size'])? '': ' gdlr-core-style-' . $args['item-size']; 
				$additional_class .= empty($args['blog-widget-style'])? ' gdlr-core-style-1': ' gdlr-core-' . $args['blog-widget-style']; 
				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-widget gdlr-core-item-mglr clearfix ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style(array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				)) . ' >';
				if( empty($args['show-thumbnail']) || $args['show-thumbnail'] == 'enable' ){
					$ret .= $this->blog_thumbnail(array(
						'thumbnail-size' => ($featured)? $args['thumbnail-size']: 'thumbnail',
						'opacity-on-hover' => empty($args['enable-thumbnail-opacity-on-hover'])? 'enable': $args['enable-thumbnail-opacity-on-hover'],
						'zoom-on-hover' => empty($args['enable-thumbnail-zoom-on-hover'])? 'enable': $args['enable-thumbnail-zoom-on-hover'],
						'grayscale-effect' => empty($args['enable-thumbnail-grayscale-effect'])? 'disable': $args['enable-thumbnail-grayscale-effect']
					), array(
						'border-radius' => empty($args['blog-border-radius'])? '': $args['blog-border-radius']
					));
				}

				$separator = '';
				if( !empty($args['blog-widget-style']) && $args['blog-widget-style'] == 'style-2' ){
					$separator = '•';
				}

				$ret .= '<div class="gdlr-core-blog-widget-content" >';
				$ret .= $this->blog_title($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'separator' => $separator
				));

				if( !empty($args['blog-widget-style']) && $args['blog-widget-style'] == 'style-3' ){
					$ret .= '<div class="gdlr-core-blog-widget-read-more" ><a href="' . esc_url(get_permalink()) . '" >' . esc_html__('Read More', 'goodlayers-core') . '</a></div>';
				}

				$ret .= '</div>'; // gdlr-core-blog-widget-content
				$ret .= '</div>'; // gdlr-core-blog-widget
				
				return $ret;
			} 			

			// blog list
			function blog_list( $args ){

				$with_frame = ( !empty($args['blog-list-with-frame']) && $args['blog-list-with-frame'] == 'enable' );
				$additional_class  = ($args['blog-style'] == 'blog-list-center')? ' gdlr-core-center-align': '';
				$additional_class .= ($with_frame)? ' gdlr-core-blog-list-with-frame': '';
				
				$ret  = '<div class="gdlr-core-item-list gdlr-core-blog-list gdlr-core-item-pdlr ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style(array(
					'margin-bottom' => empty($args['margin-bottom'])? '': $args['margin-bottom']
				)) . ' >';
				$ret .= $with_frame? '<div class="gdlr-core-blog-list-frame gdlr-core-skin-e-background">': '';
				$ret .= $this->blog_title($args);
				$ret .= $this->blog_info(array(
					'display' => $args['meta-option'],
					'wrapper' => true,
					'icon' => false,
					'separator' => '/'
				));
				$ret .= $with_frame? '</div>': '';
				$ret .= '</div>'; // gdlr-core-blog-list
				
				return $ret;
			} 				
			
		} // gdlr_core_blog_item
	} // class_exists
	
	
	
	
	
	
	
	
	
	
	
	
	
	
	
	
	
	