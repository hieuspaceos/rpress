<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	if( !class_exists('gdlr_core_pb_wrapper_sidebar') ){
		class gdlr_core_pb_wrapper_sidebar{
			
			// get the element settings
			static function get_settings(){
				return array(
					'feature' => true,
					'type' => 'sidebar',
					'title' => esc_html__('Sidebar Wrapper', 'goodlayers-core'),
					'thumbnail' => GDLR_CORE_URL . '/framework/images/page-builder/wrapper-sidebar.png',
				);
			}
			
			// return the element options
			static function get_options(){
				return apply_filters('gdlr_core_pb_wrapper_sidebar_options', array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'content-layout' => array(
								'title' => esc_html__('Content Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'boxed' => esc_html__('Boxed ( content stays within container area )', 'goodlayers-core'),
									'full' => esc_html__('Full', 'goodlayers-core'),
								)
							),
							'sidebar' => array(
								'title' => esc_html__('Sidebar', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'sidebar',
								'without-none' => true,
								'default' => 'right',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'sidebar-left' => array(
								'title' => esc_html__('Sidebar Left', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'sidebar',
								'condition' => array( 'sidebar' => array('left', 'both') )
							),
							'sidebar-right' => array(
								'title' => esc_html__('Sidebar Right', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'sidebar',
								'condition' => array( 'sidebar' => array('right', 'both') )
							),
							'sidebar-width' => array(
								'title' => esc_html__('Sidebar Width', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 30 => '1/2', 20 => '1/3', 15 => '1/4', 12 => '1/5', 10 => '1/6', 18 => '30%' ),
								'default' => 20
							),
							'enable-sticky-sidebar' => array(
								'title' => esc_html__('Enable Sticky Sidebar', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('This option only could only works when you\'re not assigning the background. Effects will only be shown on front end of the site', 'goodlayers-core')
							),
							'hide-this-wrapper-in' => array(
								'title' => esc_html__('Hide This Wrapper In', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'desktop' => esc_html__('Desktop', 'goodlayers-core'),
									'desktop-tablet' => esc_html__('Desktop & Tablet', 'goodlayers-core'),
									'tablet' => esc_html__('Tablet', 'goodlayers-core'),
									'tablet-mobile' => esc_html__('Tablet & Mobile', 'goodlayers-core'),
									'mobile' => esc_html__('Mobile', 'goodlayers-core'),
								)
							),
							'margin' => array(
								'title' => esc_html__('Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
								'description' => esc_html__('Spaces before/after this wrapper. ( outside the background area )', 'goodlayers-core')
							),
						),
					),
					'background' => array(
						'title' => esc_html__('Content Background', 'goodlayers-core'),
						'options' => array(
							'background-area' => array(
								'title' => esc_html__('Background Area', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Content', 'goodlayers-core'),
									'item' => esc_html__('Item', 'goodlayers-core')
								)
							),
							'padding' => array(
								'title' => esc_html__('Content Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' ),
								'description' => esc_html__('Spaces before/after the item inside it. ( including the background area )', 'goodlayers-core')
							),
							'background-type' => array(
								'title' => esc_html__('Background Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'color' => esc_html__('Color', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
									'video' => esc_html__('Video ( Youtube & Vimeo )', 'goodlayers-core'),
									'html5-video' => esc_html__('Html5 Video', 'goodlayers-core'),
									'pattern' => esc_html__('Pattern', 'goodlayers-core'),
								),
								'default' => 'color'
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'background-type' => 'color' )
							),
							'background-image' => array(
								'title' => esc_html__('Background Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-image-style' => array(
								'title' => esc_html__('Background Image Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'cover' => esc_html__('Cover ( full width and height )', 'goodlayers-core'),
									'no-repeat' => esc_html__('No Repeat', 'goodlayers-core'),
									'repeat' => esc_html__('Repeat X & Y', 'goodlayers-core'),
									'repeat-x' => esc_html__('Repeat X', 'goodlayers-core'),
									'repeat-y' => esc_html__('Repeat Y', 'goodlayers-core'),
								),
								'default' => 'cover',
								'condition' => array( 'background-type' => 'image' )
							), 
							'background-image-position' => array(
								'title' => esc_html__('Background Image Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'center' => esc_html__('Center', 'goodlayers-core'),
									'top-left' => esc_html__('Top Left', 'goodlayers-core'),
									'top-center' => esc_html__('Top Center', 'goodlayers-core'),
									'top-right' => esc_html__('Top Right', 'goodlayers-core'),
									'center-left' => esc_html__('Center Left', 'goodlayers-core'),
									'center-right' => esc_html__('Center Right', 'goodlayers-core'),
									'bottom-left' => esc_html__('Bottom Left', 'goodlayers-core'),
									'bottom-center' => esc_html__('Bottom Center', 'goodlayers-core'),
									'bottom-right' => esc_html__('Bottom Right', 'goodlayers-core'),
								),
								'default' => 'center',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-video-url' => array(
								'title' => esc_html__('Background Video URL', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'video' ),
							),
							'background-video-url-mp4' => array(
								'title' => esc_html__('Background Video URL (MP4)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-url-webm' => array(
								'title' => esc_html__('Background Video URL (WEBM)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-url-ogg' => array(
								'title' => esc_html__('Background Video URL (ogg)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-image' => array(
								'title' => esc_html__('Background Image Fallback', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'background-type' => array('video', 'html5-video') ),
								'description' => esc_html__('This background will be showing up when the device you\'re using cannot render the video as background ( eg. mobile device )', 'goodlayers-core'),
							),
							'background-pattern' => array(
								'title' => esc_html__('Background Pattern', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'pattern',
								'default' => 'pattern-1',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'background-type' => 'pattern' )
							), 
							'pattern-opacity' => array(
								'title' => esc_html__('Pattern Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => 1,
								'condition' => array( 'background-type' => 'pattern' ), 
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							), 
							'parallax-speed' => array(
								'title' => esc_html__('Parallax Speed', 'goodlayers-core'),
								'type' => 'text',
								'default' => 0.8,
								'condition' => array( 'background-type' => array('image', 'video', 'html5-video', 'pattern') ), 
								'description' => esc_html__('Fill the number between -1 to 1, ( value 1 equals to background-attachment: fixed )', 'goodlayers-core')
							),
							'skin' => array(
								'title' => esc_html__('Skin', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'skin'
							),
						),
					),
					'sidebar-background' => array(
						'title' => esc_html__('Sidebar Background', 'goodlayers-core'),
						'options' => array(
							'sidebar-padding' => array(
								'title' => esc_html__('Sidebar Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' ),
								'description' => esc_html__('Spaces before/after the item inside it. ( including the background area )', 'goodlayers-core')
							),
							'sidebar-background-color' => array(
								'title' => esc_html__('Sidebar Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'sticky-sidebar-top-offset' => array(
								'title' => esc_html__('Sticky Sidebar Top Offset', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'sidebar-skin' => array(
								'title' => esc_html__('Sidebar Skin', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'skin'
							),
						),
					),
					'tablet-layout' => array(
						'title' => esc_html__('Tablet Layout', 'goodlayers-core'),
						'options' => array(
							'tablet-breaks-at' => array(
								'title' => esc_html__('Breaks Sidebar/Content Layout At', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'description' => esc_html__('Fill number > 999px. This option will not effects the "Hide wrapper in" option.', 'goodlayers-core')
							),
							'tablet-padding' => array(
								'title' => esc_html__('Tablet Content Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
							'tablet-sidebar-padding' => array(
								'title' => esc_html__('Tablet Sidebar Content Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
						)
					),
					'mobile-layout' => array(
						'title' => esc_html__('Mobile Layout', 'goodlayers-core'),
						'options' => array(
							'mobile-breaks-at' => array(
								'title' => esc_html__('Breaks Sidebar/Content Layout At', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '767px',
								'description' => esc_html__('Fill number > 767px. This option will not effects the "Hide wrapper in" option.', 'goodlayers-core')
							),
							'mobile-content-order' => array(
								'title' => esc_html__('Mobile Content Order', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Only fill number here', 'goodlayers-core')
							),
							'mobile-sidebar-order' => array(
								'title' => esc_html__('Mobile Sidebar Order', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Only fill number here', 'goodlayers-core')
							),
							'mobile-padding' => array(
								'title' => esc_html__('Mobile Content Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
							'mobile-sidebar-padding' => array(
								'title' => esc_html__('Mobile Sidebar Content Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' )
							),
						)
					)
				));
			}			
			
			// get element template for page builder
			static function get_template( $options = array(), $callback = '' ){
				
				$sidebar = empty($options['value']['sidebar'])? 'right': $options['value']['sidebar'];
				$sidebar_left = empty($options['value']['sidebar-left'])? '': $options['value']['sidebar-left'];
				$sidebar_right = empty($options['value']['sidebar-right'])? '': $options['value']['sidebar-right'];
				$sidebar_width = empty($options['value']['sidebar-width'])? 20: intval($options['value']['sidebar-width']);
				
				$container_width = 60;
				if( $sidebar == 'left' || $sidebar == 'right' ){ $container_width -= $sidebar_width; }
				if( $sidebar == 'both' ){ $container_width -= 2 * $sidebar_width; }
				
				$wrapper  = '<div class="gdlr-core-page-builder-wrapper gdlr-core-page-builder-sidebar-wrapper" data-template="wrapper" data-type="sidebar" ';
				$wrapper .= 'data-sidebar="' . (empty($options['value']['sidebar'])? 'right': esc_attr($options['value']['sidebar'])) . '" ';
				$wrapper .= empty($options['value'])? '': 'data-value="' . esc_attr(json_encode($options['value'])) . '" ';
				$wrapper .= '>';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-content">';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-head">';
				$wrapper .= '<span class="gdlr-core-page-builder-wrapper-head-title" >' . esc_html__('Sidebar Wrapper', 'goodlayers-core') . '</span>';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-settings">';
				$wrapper .= '<i class="fa fa-edit gdlr-core-page-builder-wrapper-edit" ></i>';
				$wrapper .= '<i class="fa fa-copy gdlr-core-page-builder-wrapper-copy" ></i>';
				$wrapper .= '<i class="fa fa-download gdlr-core-page-builder-wrapper-save" ></i>';
				$wrapper .= '<i class="fa fa-remove gdlr-core-page-builder-wrapper-remove" ></i>';
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-settings
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-head

				// container style
				$container_style_atts = array(
					'margin' => (empty($options['value']['margin']) || $options['value']['margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? '': $options['value']['margin'],
					
				);

				// container width calss
				$container_class = '';
				if( empty($options['value']['content-layout']) || $options['value']['content-layout'] == 'boxed' ){
					$container_class = 'gdlr-core-container';
				}else if( $options['value']['content-layout'] == 'full' ){
					$container_class = 'gdlr-core-pbf-sidebar-wrapper-full';
				}
				$container_class .= empty($options['value']['class'])? '': ' ' . $options['value']['class'];

				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-sidebar-container clearfix gdlr-core-line-height-0 gdlr-core-js ' . esc_attr($container_class) . '" ' . gdlr_core_esc_style($container_style_atts) . ' >';
				
				// content
				$content_class  = ' gdlr-core-column-' . esc_attr($container_width);
				$content_class .= ' gdlr-core-pbf-sidebar-padding gdlr-core-line-height';
				if( $sidebar == 'left' ){
					$content_class .= ' gdlr-core-column-extend-right';
				}else if( $sidebar == 'right' ){
					$content_class .= ' gdlr-core-column-extend-left';
				}
				if( !empty($options['value']['background-area']) && $options['value']['background-area'] == 'item' ){
					$wrapper .= self::get_background($options);
				}
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-sidebar-content ' . esc_attr($content_class) . '" ' . gdlr_core_esc_style(array(
					'padding' => (empty($options['value']['padding']) || $options['value']['padding'] == array( 
						'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
					))? '': $options['value']['padding']
				)) . ' >';
				if( empty($options['value']['background-area']) ){
					$wrapper .= self::get_background($options);
				}
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-sidebar-content-inner gdlr-core-wrapper-sortable clearfix" ';
				$wrapper .= (empty($options['value']['skin'])? '': 'data-skin="' . esc_attr($options['value']['skin']) . '" ');
				$wrapper .= ' >';
				if( !empty($options['items']) ){
					$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
				}
				$wrapper .= '</div>';
				$wrapper .= '</div>';

				$sidebar_class  = apply_filters('gdlr_core_sidebar_class', '');
				$sidebar_class .= ' gdlr-core-column-' . esc_attr($sidebar_width);
				$sidebar_class .= ' gdlr-core-pbf-sidebar-padding  gdlr-core-line-height';

				// left sidebar
				if( $sidebar == 'left' || $sidebar == 'both' ){
					$wrapper .= '<div class="gdlr-core-page-builder-wrapper-sidebar-left gdlr-core-column-extend-left ' . esc_attr($sidebar_class) . '" ' . gdlr_core_esc_style(array(
						'padding' => (empty($options['value']['sidebar-padding']) || $options['value']['sidebar-padding'] == array( 
							'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
						))? '': $options['value']['sidebar-padding']
					));
					$wrapper .= empty($options['value']['sticky-sidebar-top-offset'])? '': 'data-sticky-offset="' . esc_attr($options['value']['sticky-sidebar-top-offset']) . '" ';
					$wrapper .= ' >';
					$wrapper .= self::get_sidebar_background($options);
					$wrapper .= '<div class="gdlr-core-sidebar-inner-bg gdlr-core-item-mglr"></div>';
					$wrapper .= '</div>';
				}
				
				// right sidebar
				if( $sidebar == 'right' || $sidebar == 'both' ){
					$wrapper .= '<div class="gdlr-core-page-builder-wrapper-sidebar-right gdlr-core-column-extend-right ' . esc_attr($sidebar_class) . '" ' . gdlr_core_esc_style(array(
						'padding' => (empty($options['value']['sidebar-padding']) || $options['value']['sidebar-padding'] == array( 
							'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
						))? '': $options['value']['sidebar-padding']
					));
					$wrapper .= empty($options['value']['sticky-sidebar-top-offset'])? '': 'data-sticky-offset="' . esc_attr($options['value']['sticky-sidebar-top-offset']) . '" ';
					$wrapper .= ' >';
					$wrapper .= self::get_sidebar_background($options);
					$wrapper .= '<div class="gdlr-core-sidebar-inner-bg gdlr-core-item-mglr"></div>';
					$wrapper .= '</div>';
				}
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-sidebar-container
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content

				// script for background wrapper
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-sidebar-background-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var background_wrap = jQuery('#gdlr-core-preview-sidebar-background-<?php echo esc_attr($id); ?>').parent();
	new gdlr_core_sidebar_wrapper( background_wrap );
	background_wrap.gdlr_core_parallax_background().gdlr_core_ux();
	jQuery(window).trigger('scroll');
});
</script><?php	
				$wrapper .= ob_get_contents();
				ob_end_clean();	

				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper
				
				return $wrapper;
				
			}
			
			// get element content for front end page builder
			static function get_content( $options = array(), $callback = '' ){
				
				$sidebar = empty($options['value']['sidebar'])? 'right': $options['value']['sidebar'];
				$sidebar_left = empty($options['value']['sidebar-left'])? '': $options['value']['sidebar-left'];
				$sidebar_right = empty($options['value']['sidebar-right'])? '': $options['value']['sidebar-right'];
				$sidebar_width = empty($options['value']['sidebar-width'])? 20: intval($options['value']['sidebar-width']);
				
				$container_width = 60;
				if( $sidebar == 'left' || $sidebar == 'right' ){ $container_width -= $sidebar_width; }
				if( $sidebar == 'both' ){ $container_width -= 2 * $sidebar_width; }
				
				// device display class
				$wrapper_display_class = '';
				if( !empty($options['value']['hide-this-wrapper-in']) && $options['value']['hide-this-wrapper-in'] != 'none' ){
					$wrapper_display_class .= ' gdlr-core-hide-in-' . $options['value']['hide-this-wrapper-in'];
				}
				if( !empty($options['value']['enable-sticky-sidebar']) && $options['value']['enable-sticky-sidebar'] == 'enable' ){
					$wrapper_display_class .= ' gdlr-core-sticky-sidebar gdlr-core-js ';
				}

				// container style
				$container_style_atts = array(
					'margin' => (empty($options['value']['margin']) || $options['value']['margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? '': $options['value']['margin']
				);

				// tablet style
				$tablet_style = '';
				if( !empty($options['value']['tablet-padding']) && $options['value']['tablet-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$tablet_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-content{';
					$tablet_style .= gdlr_core_esc_style(array('padding'=>$options['value']['tablet-padding']), false, true);
					$tablet_style .= '} ';
				}
				if( !empty($options['value']['tablet-sidebar-padding']) && $options['value']['tablet-sidebar-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$tablet_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-left,';
					$tablet_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-right{';
					$tablet_style .= gdlr_core_esc_style(array('padding'=>$options['value']['tablet-sidebar-padding']), false, true);
					$tablet_style .= '} ';
				}

				$tablet_break_size = empty($options['value']['tablet-breaks-at'])? '999px': $options['value']['tablet-breaks-at'];

				if( !empty($tablet_style) ){
					$tablet_style = '@media only screen and (max-width: ' . $tablet_break_size . '){' . $tablet_style . '}';
				}

				// mobile style
				$mobile_style = '';
				if( !empty($options['value']['mobile-padding']) && $options['value']['mobile-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-content{';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['mobile-padding']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['mobile-sidebar-padding']) && $options['value']['mobile-sidebar-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-left,';
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-right{';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['mobile-sidebar-padding']), false, true);
					$mobile_style .= '} ';
				}

				$mobile_break_size = empty($options['value']['mobile-breaks-at'])? '767px': $options['value']['mobile-breaks-at'];
				if( $mobile_break_size != '767px' ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-container{ display:flex; flex-direction: column }';
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-content,';
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-sidebar-wrapper .gdlr-core-pbf-sidebar-right{ float:none; width: 100%; }';
				}

				if( !empty($mobile_style) ){
					$mobile_style = '@media only screen and (max-width: ' . $mobile_break_size . '){' . $mobile_style . '}';
				}

				if( $mobile_style || $tablet_style ){
					$extra_css = $tablet_style . $mobile_style;
					if( empty($options['value']['id']) ){
						global $gdlr_core_wrapper_id;
						$gdlr_core_wrapper_id = empty($gdlr_core_wrapper_id)? 1: $gdlr_core_wrapper_id + 1;
						$options['value']['id'] = 'gdlr-core-sidebar-wrapper-' . $gdlr_core_wrapper_id;
					}
					$extra_css = str_replace('gdlr-core-temp-id', $options['value']['id'], $extra_css);
					gdlr_core_add_inline_style($extra_css);
				}

				// container width calss
				$container_class = '';
				if( empty($options['value']['content-layout']) || $options['value']['content-layout'] == 'boxed' ){
					$container_class = 'gdlr-core-container';
				}else if( $options['value']['content-layout'] == 'full' ){
					$container_class = 'gdlr-core-pbf-sidebar-wrapper-full';
				}
				$container_class .= empty($options['value']['class'])? '': ' ' . $options['value']['class'];

				$wrapper  = '<div class="gdlr-core-pbf-sidebar-wrapper ' . esc_attr($wrapper_display_class) . '" ';
				$wrapper .= (empty($options['value']['id'])? '': 'id="' . esc_attr($options['value']['id']) . '" ');
				$wrapper .= gdlr_core_esc_style($container_style_atts) . ' >';
				$wrapper .= '<div class="gdlr-core-pbf-sidebar-container gdlr-core-line-height-0 clearfix gdlr-core-js ' . esc_attr($container_class) . '">';
				
				// content
				$content_class  = ' gdlr-core-column-' . esc_attr($container_width);
				$content_class .= ' gdlr-core-pbf-sidebar-padding gdlr-core-line-height';
				if( $sidebar == 'left' ){
					$content_class .= ' gdlr-core-column-extend-right';
				}else if( $sidebar == 'right' ){
					$content_class .= ' gdlr-core-column-extend-left';
				}

				if( !empty($options['value']['background-area']) && $options['value']['background-area'] == 'item' ){
					$wrapper .= self::get_background($options);
				}
				$wrapper .= '<div class="gdlr-core-pbf-sidebar-content ' . esc_attr($content_class) . '" ' . gdlr_core_esc_style(array(
					'padding' => (empty($options['value']['padding']) || $options['value']['padding'] == array( 
						'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
					))? '': $options['value']['padding'],
					'order' => empty($options['value']['mobile-content-order'])? '': $options['value']['mobile-content-order']
				)) . ' >';
				if( empty($options['value']['background-area']) ){
					$wrapper .= self::get_background($options);
				}
				if( !empty($options['items']) ){
					$wrapper .= '<div class="gdlr-core-pbf-sidebar-content-inner" ';
					$wrapper .= (empty($options['value']['skin'])? '': 'data-skin="' . esc_attr($options['value']['skin']) . '" ');
					$wrapper .= ' >';
					$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
					$wrapper .= '</div>'; // gdlr-core-pbf-sidebar-content-inner
				}
				$wrapper .= '</div>'; // gdlr-core-pbf-sidebar-content
				
				$sidebar_class  = apply_filters('gdlr_core_sidebar_class', '');
				$sidebar_class .= ' gdlr-core-column-' . esc_attr($sidebar_width);
				$sidebar_class .= ' gdlr-core-pbf-sidebar-padding  gdlr-core-line-height';

				// left sidebar
				if( $sidebar == 'left' || $sidebar == 'both' ){
					$wrapper .= '<div class="gdlr-core-pbf-sidebar-left gdlr-core-column-extend-left ' . esc_attr($sidebar_class) .'" ';
					$wrapper .= (empty($options['value']['sidebar-skin'])? '': 'data-skin="' . esc_attr($options['value']['sidebar-skin']) . '" ');
					$wrapper .= gdlr_core_esc_style(array(
						'padding' => (empty($options['value']['sidebar-padding']) || $options['value']['sidebar-padding'] == array( 
							'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
						))? '': $options['value']['sidebar-padding'],
						'order' => empty($options['value']['mobile-sidebar-order'])? '': $options['value']['mobile-sidebar-order']
					));
					$wrapper .= empty($options['value']['sticky-sidebar-top-offset'])? '': 'data-sticky-offset="' . esc_attr($options['value']['sticky-sidebar-top-offset']) . '" ';
					$wrapper .= ' >';
					$wrapper .= self::get_sidebar_background($options);
					$wrapper .= '<div class="gdlr-core-sidebar-item gdlr-core-item-pdlr">';
					ob_start();
					if( !empty($options['value']) ){
						do_action('gdlr_core_pb_wrapper_sidebar_left_content', $options['value']);
					}
					if( is_active_sidebar($sidebar_left) ){
						dynamic_sidebar($sidebar_left);
					}
					$wrapper .= ob_get_contents();
					ob_end_clean();
					$wrapper .= '</div>';
					$wrapper .= '</div>';
				}
				
				// right sidebar
				if( $sidebar == 'right' || $sidebar == 'both' ){
					$wrapper .= '<div class="gdlr-core-pbf-sidebar-right gdlr-core-column-extend-right ' . esc_attr($sidebar_class) .'" ';
					$wrapper .= (empty($options['value']['sidebar-skin'])? '': 'data-skin="' . esc_attr($options['value']['sidebar-skin']) . '" ');
					$wrapper .= gdlr_core_esc_style(array(
						'padding' => (empty($options['value']['sidebar-padding']) || $options['value']['sidebar-padding'] == array( 
							'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
						))? '': $options['value']['sidebar-padding'],
						'order' => empty($options['value']['mobile-sidebar-order'])? '': $options['value']['mobile-sidebar-order']
					));
					$wrapper .= empty($options['value']['sticky-sidebar-top-offset'])? '': 'data-sticky-offset="' . esc_attr($options['value']['sticky-sidebar-top-offset']) . '" ';
					$wrapper .= ' >';
					$wrapper .= self::get_sidebar_background($options);
					$wrapper .= '<div class="gdlr-core-sidebar-item gdlr-core-item-pdlr">';

					ob_start();
					if( !empty($options['value']) ){
						do_action('gdlr_core_pb_wrapper_sidebar_right_content', $options['value']);
					}
					if( is_active_sidebar($sidebar_right) ){
						dynamic_sidebar($sidebar_right);
					}
					$wrapper .= ob_get_contents();
					ob_end_clean();
					$wrapper .= '</div>';
					$wrapper .= '</div>';
				}
				$wrapper .= '</div>'; // gdlr-core-pbf-sidebar-container
				$wrapper .= '</div>'; // gdlr-core-pbf-sidebar-wrapper
				
				return $wrapper;				
				
			}

			// get background
			static function get_sidebar_background( $options ){

				$wrapper_background = '';
				if( !empty($options['value']['sidebar-background-color']) ){
					$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
						'background-color' => $options['value']['sidebar-background-color']
					)) . ' ></div>';
				}


				return $wrapper_background;
			}

			// get background
			static function get_background( $options ){

				// for background
				$wrapper_background = '';
				if( !empty($options['value']['background-type']) ){
					
					// color background
					if( $options['value']['background-type'] == 'color' ){
						
						if( !empty($options['value']['background-color']) ){
							$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
								'background-color' => $options['value']['background-color']
							)) . ' ></div>';
						}
					
					// image background
					}else if( $options['value']['background-type'] == 'image' ){
						$bgi_atts = array(
							'background-image' => empty($options['value']['background-image'])? '': $options['value']['background-image']
						);
						if( !empty($options['value']['background-image-style']) ){
							if( $options['value']['background-image-style'] == 'cover' ){
								$bgi_atts['background-size'] = 'cover';
							}else{
								$bgi_atts['background-repeat'] = $options['value']['background-image-style'];
							}
							if( !empty($options['value']['background-image-position']) ){
								$bgi_atts['background-position'] = str_replace('-', ' ', $options['value']['background-image-position']);
							}
						}
						
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap">';
						$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
						$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
						$wrapper_background .= ' ></div>';
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// pattern background
					}else if( $options['value']['background-type'] == 'pattern' ){
						$bgi_atts = array(
							'background-image' => GDLR_CORE_URL . '/include/images/pattern/' . (empty($options['value']['background-pattern'])? 'pattern-1': $options['value']['background-pattern']) . '.png',
							'background-repeat' => 'repeat',
							'background-position' => 'center',
							'opacity' => empty($options['value']['pattern-opacity'])? '1': $options['value']['pattern-opacity'],
						);
						
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap">';
						$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
						$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
						$wrapper_background .= ' ></div>';
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// video background
					}else{
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap">';
						$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style(array(
							'background-position' => 'center',
							'background-size' => 'cover',
						));
						$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
						$wrapper_background .= ' data-video-fallback="' . (empty($options['value']['background-video-image'])? '': gdlr_core_get_image_url($options['value']['background-video-image'])) . '" ';
						$wrapper_background .= ' >';
						$wrapper_background .= '<div class="gdlr-core-pbf-background-video" data-background-type="video" >';
						
						if( $options['value']['background-type'] == 'video' ){
							if( !empty($options['value']['background-video-url']) ){
								$wrapper_background .= gdlr_core_get_video(
									$options['value']['background-video-url'], 
									array('width' => '100%', 'height' => '100%'), 
									array('background' => 1)
								);
							}
						}else if( $options['value']['background-type'] == 'html5-video' ){
							$wrapper_background .= '<video autoplay loop muted >';
							if( $options['value']['background-video-url-mp4'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-mp4']) . '" type="video/mp4">';
							}
							if( $options['value']['background-video-url-webm'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-webm']) . '" type="video/webm">';
							}
							if( $options['value']['background-video-url-ogg'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-ogg']) . '" type="video/ogg">';
							}
							$wrapper_background .= '</video>';
						}
						$wrapper_background .= '</div>';
						$wrapper_background .= '</div>'; // gdlr-cpre-pbc-background
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					}
				}

				return $wrapper_background;

			} // get_background			
			
		} // gdlr_core_pb_wrapper_sidebar
	} // class_exists	